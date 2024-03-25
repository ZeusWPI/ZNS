use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::UdpSocket;

use crate::db::models::{get_from_database, insert_into_database};
use crate::parser::FromBytes;
use crate::structs::{Class, Message, Type, RCODE, RR, Opcode};
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
        let answer = get_from_database(&question).await;

        match answer {
            Ok(rr) => {
                response.header.flags = set_response_flags(response.header.flags, RCODE::NOERROR);
                response.header.ancount = 1;
                response.answer = vec![rr]
            }
            Err(e) => {
                response.header.flags = set_response_flags(response.header.flags, RCODE::NXDOMAIN);
                eprintln!("{}", e);
            }
        }
    }

    response
}

async fn handle_update(message: Message) -> Message {
    let mut response = message.clone();

    // Zone section (question) processing
    if (message.header.qdcount != 1) || !matches!(message.question[0].qtype, Type::SOA) {
        response.header.flags = set_response_flags(response.header.flags, RCODE::FORMERR);
        return response;
    }

    // Check Zone authority
    let zone = &message.question[0];
    let zlen = zone.qname.len();
    if !(zlen >= 2
        && zone.qname[zlen - 1] == "gent"
        && zone.qname[zlen - 2] == "zeus")
    {
        response.header.flags = set_response_flags(response.header.flags, RCODE::NOTAUTH);
        return response;
    }

    // Check Prerequisite    TODO: implement this
    if message.header.ancount > 0 {
        response.header.flags = set_response_flags(response.header.flags, RCODE::NOTIMP);
        return response;
    }

    // Check Requestor Permission
    //  TODO: implement this, use rfc2931

    // Update Section Prescan
    for rr in &message.authority {
        let rlen = rr.name.len();

        // Check if rr has same zone
        if rlen < zlen || !(vec_equal(&zone.qname, &rr.name[rlen - zlen..])) {
            response.header.flags = set_response_flags(response.header.flags, RCODE::NOTZONE);
            return response;
        }

        if (rr.class == Class::ANY && (rr.ttl != 0 || rr.rdlength != 0))
            || (rr.class == Class::NONE && rr.ttl != 0)
            || rr.class != zone.qclass
        {
            response.header.flags = set_response_flags(response.header.flags, RCODE::FORMERR);
            return response;
        }

    }

    for rr in message.authority {
        if rr.class == zone.qclass {
            insert_into_database(rr).await;
        } else if rr.class == Class::ANY {
            response.header.flags = set_response_flags(response.header.flags, RCODE::NOTIMP);
            return response;
        } else if rr.class == Class::ANY {
            response.header.flags = set_response_flags(response.header.flags, RCODE::NOTIMP);
            return response;
        }
    }

    response.header.flags = set_response_flags(response.header.flags, RCODE::NOERROR);
    response
}

async fn get_response(bytes: &[u8]) -> Message {
    let mut i: usize = 0;
    match Message::from_bytes(bytes, &mut i) {
        Ok(message) => match get_opcode(&message.header.flags) {
            Ok(opcode) => match opcode {
                Opcode::QUERY => handle_query(message).await,
                Opcode::UPDATE => handle_update(message).await,
            },
            Err(_) => todo!(),
        },
        Err(err) => {
            println!("{}", err);
            unimplemented!() //TODO: implement this
        }
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
