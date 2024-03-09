use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::UdpSocket;

use crate::db::models::get_from_database;
use crate::parser::FromBytes;
use crate::structs::Message;

const MAX_DATAGRAM_SIZE: usize = 40_96;

async fn create_query(message: Message) -> Message {
    let mut response = message.clone();

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

pub async fn resolver_listener_loop(addr: SocketAddr) -> Result<(), Box<dyn Error>> {
    let socket_shared = Arc::new(UdpSocket::bind(addr).await?);
    loop {
        let mut data = vec![0u8; MAX_DATAGRAM_SIZE];
        let (len, addr) = socket_shared.recv_from(&mut data).await?;
        match Message::from_bytes(&data[..len]) {
            Ok(message) => {
                let socket = socket_shared.clone();
                tokio::spawn(async move {
                    let response = create_query(message).await;
                    let _ = socket
                        .send_to(Message::to_bytes(response).as_slice(), addr)
                        .await;
                });
            }
            Err(err) => println!("{}", err),
        };
    }
}
