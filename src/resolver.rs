use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::UdpSocket;

use crate::authenticate::authenticate;
use crate::db::models::{delete_from_database, get_from_database, insert_into_database};
use crate::errors::ParseError;
use crate::parser::FromBytes;
use crate::reader::Reader;
use crate::sig::Sig;
use crate::structs::{Class, Header, Message, Opcode, RRClass, RRType, Type, RCODE};
use crate::utils::vec_equal;

const MAX_DATAGRAM_SIZE: usize = 4096;

fn set_response_flags(flags: u16, rcode: RCODE) -> u16 {
    (flags | 0b1_0000_1_0_0_0_000_0000 | rcode as u16) & 0b1_1111_1_0_1_0_111_1111
}

fn get_opcode(flags: &u16) -> Result<Opcode, String> {
    Opcode::try_from((flags & 0b0111100000000000) >> 11)
}

async fn handle_query(message: Message) -> Message {
    let mut response = message.clone();
    response.header.arcount = 0; //TODO: fix this, handle unknown class values

    for question in message.question {
        let answers = get_from_database(&question).await;

        match answers {
            Ok(rrs) => {
                response.header.flags = set_response_flags(response.header.flags, RCODE::NOERROR);
                response.header.ancount = rrs.len() as u16;
                response.answer = rrs
            }
            Err(e) => {
                response.header.flags = set_response_flags(response.header.flags, RCODE::NXDOMAIN);
                eprintln!("{}", e);
            }
        }
    }

    response
}

async fn handle_update(message: Message, bytes: &[u8]) -> Message {
    let mut response = message.clone();

    // Zone section (question) processing
    if (message.header.qdcount != 1)
        || !matches!(message.question[0].qtype, Type::Type(RRType::SOA))
    {
        response.header.flags = set_response_flags(response.header.flags, RCODE::FORMERR);
        return response;
    }

    // Check Zone authority
    let zone = &message.question[0];
    let zlen = zone.qname.len();
    if !(zlen >= 2 && zone.qname[zlen - 1] == "gent" && zone.qname[zlen - 2] == "zeus") {
        response.header.flags = set_response_flags(response.header.flags, RCODE::NOTAUTH);
        return response;
    }

    // Check Prerequisite    TODO: implement this

    //TODO: this code is ugly
    let last = message.additional.last();
    if last.is_some() && last.unwrap()._type == Type::Type(RRType::KEY) {
        let sig = Sig::new(last.unwrap(), bytes);

        if !authenticate(&sig, &zone.qname).await.is_ok_and(|x| x) {
            response.header.flags = set_response_flags(response.header.flags, RCODE::NOTAUTH);
            return response;
        }
    } else {
        response.header.flags = set_response_flags(response.header.flags, RCODE::NOTAUTH);
        return response;
    }

    // Update Section Prescan
    for rr in &message.authority {
        let rlen = rr.name.len();

        // Check if rr has same zone
        if rlen < zlen || !(vec_equal(&zone.qname, &rr.name[rlen - zlen..])) {
            response.header.flags = set_response_flags(response.header.flags, RCODE::NOTZONE);
            return response;
        }

        if (rr.class == Class::Class(RRClass::ANY) && (rr.ttl != 0 || rr.rdlength != 0))
            || (rr.class == Class::Class(RRClass::NONE) && rr.ttl != 0)
            || ![
                Class::Class(RRClass::NONE),
                Class::Class(RRClass::ANY),
                zone.qclass.clone(),
            ]
            .contains(&rr.class)
        {
            response.header.flags = set_response_flags(response.header.flags, RCODE::FORMERR);
            return response;
        }
    }

    for rr in message.authority {
        if rr.class == zone.qclass {
            let _ = insert_into_database(rr).await;
        } else if rr.class == Class::Class(RRClass::ANY) {
            if rr._type == Type::Type(RRType::ANY) {
                if rr.name == zone.qname {
                    response.header.flags =
                        set_response_flags(response.header.flags, RCODE::NOTIMP);
                    return response;
                } else {
                    delete_from_database(rr.name, None, Class::Class(RRClass::IN), None).await;
                }
            } else {
                delete_from_database(rr.name, Some(rr._type), Class::Class(RRClass::IN), None)
                    .await;
            }
        } else if rr.class == Class::Class(RRClass::NONE) {
            if rr._type == Type::Type(RRType::SOA) {
                continue;
            }
            delete_from_database(
                rr.name,
                Some(rr._type),
                Class::Class(RRClass::IN),
                Some(rr.rdata),
            )
            .await;
        }
    }

    response.header.flags = set_response_flags(response.header.flags, RCODE::NOERROR);
    response
}

fn handle_parse_error(bytes: &[u8], err: ParseError) -> Message {
    eprintln!("{}", err);
    let mut reader = Reader::new(bytes);
    let mut header = Header::from_bytes(&mut reader).unwrap_or(Header {
        id: 0,
        flags: 0,
        qdcount: 0,
        ancount: 0,
        nscount: 0,
        arcount: 0,
    });

    header.qdcount = 0;
    header.ancount = 0;
    header.nscount = 0;
    header.arcount = 0;
    header.flags = set_response_flags(header.flags, RCODE::FORMERR);

    Message {
        header,
        question: vec![],
        answer: vec![],
        authority: vec![],
        additional: vec![],
    }
}

async fn get_response(bytes: &[u8]) -> Message {
    let mut reader = Reader::new(bytes);
    match Message::from_bytes(&mut reader) {
        Ok(message) => match get_opcode(&message.header.flags) {
            Ok(opcode) => match opcode {
                Opcode::QUERY => handle_query(message).await,
                Opcode::UPDATE => handle_update(message, bytes).await,
            },
            Err(_) => todo!(),
        },
        Err(err) => handle_parse_error(bytes, err),
    }
}

pub async fn resolver_listener_loop(addr: SocketAddr) -> Result<(), Box<dyn Error>> {
    let socket_shared = Arc::new(UdpSocket::bind(addr).await?);
    loop {
        let mut data = vec![0u8; MAX_DATAGRAM_SIZE];
        let (len, addr) = socket_shared.recv_from(&mut data).await?;
        let socket = socket_shared.clone();
        tokio::spawn(async move {
            let response = get_response(&data[..len]).await;
            let _ = socket
                .send_to(Message::to_bytes(response).as_slice(), addr)
                .await;
        });
    }
}
