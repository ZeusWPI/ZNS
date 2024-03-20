use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::UdpSocket;

use crate::db::models::get_from_database;
use crate::parser::FromBytes;
use crate::structs::{Class, Message, Type, RCODE};
use crate::utils::vec_equal;

const MAX_DATAGRAM_SIZE: usize = 4096;

fn set_response_flags(flags: u16, rcode: RCODE) -> u16 {
    (flags | 0b1000010000000000 | rcode as u16) & 0b1_1111_1_0_1_0_111_1111
}

async fn handle_query(message: Message) -> Message {
    let mut response = message.clone();

    for question in message.question {
        let answer = get_from_database(&question).await;

        match answer {
            Ok(rr) => {
                response.header.flags = set_response_flags(response.header.flags, RCODE::NOERROR);
                response.header.ancount = 1;
                response.answer = vec![rr]
            }
            Err(e) => {
                response.header.flags |= 0b1000010110000011;
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
    let zlen = message.question[0].qname.len();
    if !(zlen >= 2
        && message.question[0].qname[zlen - 1] == "gent"
        && message.question[0].qname[zlen - 2] == "zeus")
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
    for rr in message.authority {
        let rlen = rr.name.len();

        // Check if rr has same zone
        if rlen < zlen || !(vec_equal(&message.question[0].qname, &rr.name[rlen - zlen..])) {
            response.header.flags = set_response_flags(response.header.flags, RCODE::NOTZONE);
            return response;
        }

        if (rr.class == Class::ANY && (rr.ttl != 0 || rr.rdlength != 0))
            || (rr.class == Class::NONE && rr.ttl != 0)
            || rr.class != message.question[0].qclass
        {
            response.header.flags = set_response_flags(response.header.flags, RCODE::FORMERR);
            return response;
        }

    }

    response
}

async fn get_response(bytes: &[u8]) -> Message {
    let mut i: usize = 0;
    match Message::from_bytes(bytes, &mut i) {
        Ok(message) => handle_query(message).await,
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
