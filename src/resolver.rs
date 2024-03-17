use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::UdpSocket;

use crate::db::models::get_from_database;
use crate::parser::{parse_opt_type, FromBytes};
use crate::structs::{Message, Type, RR};

const MAX_DATAGRAM_SIZE: usize = 4096;
const OPTION_CODE: usize = 65001;

async fn handle_normal_question(message: Message) -> Message {
    let mut response = message.clone();

    println!("{:#?}",message.question);
    let answer = get_from_database(message.question).await;
    response.header.arcount = 0;

    match answer {
        Ok(rr) => {
            response.header.flags |= 0b1000010110000000;
            response.header.ancount = 1;
            response.answer = Some(rr)
        }
        Err(e) => {
            response.header.flags |= 0b1000010110000011;
            eprintln!("{}", e);
        }
    }

    response
}

async fn handle_opt_rr(rr: RR) {
    let pairs = parse_opt_type(&rr.rdata);
    println!("{:#?}", pairs)
}

async fn get_response(message: Message) -> Message {
    match message.question.qtype {
        Type::OPT => handle_normal_question(message),
        _ => handle_normal_question(message),
    }
    .await
}

pub async fn resolver_listener_loop(addr: SocketAddr) -> Result<(), Box<dyn Error>> {
    let socket_shared = Arc::new(UdpSocket::bind(addr).await?);
    loop {
        let mut data = vec![0u8; MAX_DATAGRAM_SIZE];
        let (len, addr) = socket_shared.recv_from(&mut data).await?;
        let mut i: usize = 0;
        match Message::from_bytes(&data[..len], &mut i) {
            Ok(message) => {
                let socket = socket_shared.clone();
                tokio::spawn(async move {
                    let response = get_response(message).await;
                    let _ = socket
                        .send_to(Message::to_bytes(response).as_slice(), addr)
                        .await;
                });
            }
            Err(err) => println!("{}", err),
        };
    }
}
