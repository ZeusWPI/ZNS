use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::UdpSocket;

use crate::authenticate::authenticate;
use crate::db::models::{delete_from_database, get_from_database, insert_into_database};
use crate::errors::{DNSError, ParseError};
use crate::parser::FromBytes;
use crate::reader::Reader;
use crate::sig::Sig;
use crate::structs::{Class, Header, Message, Opcode, RRClass, RRType, Type, RCODE};
use crate::utils::vec_equal;

const MAX_DATAGRAM_SIZE: usize = 4096;

fn set_response_flags(flags: &u16, rcode: RCODE) -> u16 {
    (flags | 0b1_0000_1_0_0_0_000_0000 | rcode as u16) & 0b1_1111_1_0_1_0_111_1111
}

fn get_opcode(flags: &u16) -> Result<Opcode, String> {
    Opcode::try_from((flags & 0b0111100000000000) >> 11)
}

async fn handle_query(message: &Message) -> Result<Message, DNSError> {
    let mut response = message.clone();
    response.header.arcount = 0; //TODO: fix this, handle unknown class values

    for question in &message.question {
        let answers = get_from_database(&question).await;

        match answers {
            Ok(rrs) => {
                response.header.ancount = rrs.len() as u16;
                response.answer.extend(rrs)
            }
            Err(e) => {
                return Err(DNSError {
                    rcode: RCODE::NXDOMAIN,
                    message: e.to_string(),
                })
            }
        }
    }

    Ok(response)
}

async fn handle_update(message: &Message, bytes: &[u8]) -> Result<Message, DNSError> {
    let response = message.clone();
    // Zone section (question) processing
    if (message.header.qdcount != 1)
        || !matches!(message.question[0].qtype, Type::Type(RRType::SOA))
    {
        return Err(DNSError {
            message: "Qdcount not one".to_string(),
            rcode: RCODE::FORMERR,
        });
    }

    // Check Zone authority
    let zone = &message.question[0];
    let zlen = zone.qname.len();
    if !(zlen >= 2 && zone.qname[zlen - 1] == "gent" && zone.qname[zlen - 2] == "zeus") {
        return Err(DNSError {
            message: "Invalid zone".to_string(),
            rcode: RCODE::NOTAUTH,
        });
    }

    // Check Prerequisite    TODO: implement this

    //TODO: this code is ugly
    let last = message.additional.last();
    if last.is_some() && last.unwrap()._type == Type::Type(RRType::KEY) {
        let sig = Sig::new(last.unwrap(), bytes)?;

        if !authenticate(&sig, &zone.qname).await.is_ok_and(|x| x) {
            return Err(DNSError {
                message: "Unable to verify authentication".to_string(),
                rcode: RCODE::NOTAUTH,
            });
        }
    } else {
        return Err(DNSError {
            message: "No KEY record at the end of request found".to_string(),
            rcode: RCODE::NOTAUTH,
        });
    }

    // Update Section Prescan
    for rr in &message.authority {
        let rlen = rr.name.len();

        // Check if rr has same zone
        if rlen < zlen || !(vec_equal(&zone.qname, &rr.name[rlen - zlen..])) {
            return Err(DNSError {
                message: "RR has different zone from Question".to_string(),
                rcode: RCODE::NOTZONE,
            });
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
            return Err(DNSError {
                message: "RR has invalid rr,ttl or class".to_string(),
                rcode: RCODE::FORMERR,
            });
        }
    }

    for rr in &message.authority {
        if rr.class == zone.qclass {
            let _ = insert_into_database(&rr).await;
        } else if rr.class == Class::Class(RRClass::ANY) {
            if rr._type == Type::Type(RRType::ANY) {
                if rr.name == zone.qname {
                    return Err(DNSError {
                        message: "Not yet implemented".to_string(),
                        rcode: RCODE::NOTIMP,
                    });
                } else {
                    delete_from_database(&rr.name, None, Class::Class(RRClass::IN), None).await;
                }
            } else {
                delete_from_database(
                    &rr.name,
                    Some(rr._type.clone()),
                    Class::Class(RRClass::IN),
                    None,
                )
                .await;
            }
        } else if rr.class == Class::Class(RRClass::NONE) {
            if rr._type == Type::Type(RRType::SOA) {
                continue;
            }
            delete_from_database(
                &rr.name,
                Some(rr._type.clone()),
                Class::Class(RRClass::IN),
                Some(rr.rdata.clone()),
            )
            .await;
        }
    }

    Ok(response)
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
    header.flags = set_response_flags(&header.flags, RCODE::FORMERR);

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
        Ok(mut message) => match get_opcode(&message.header.flags) {
            Ok(opcode) => {
                let result = match opcode {
                    Opcode::QUERY => handle_query(&message).await,
                    Opcode::UPDATE => handle_update(&message, bytes).await,
                };

                match result {
                    Ok(mut response) => {
                        response.header.flags =
                            set_response_flags(&response.header.flags, RCODE::NOERROR);
                        response
                    }
                    Err(e) => {
                        eprintln!("{}", e.to_string());
                        message.header.flags = set_response_flags(&message.header.flags, e.rcode);
                        message
                    }
                }
            }
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
