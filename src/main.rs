use std::{error::Error, net::SocketAddr, sync::Arc};

use parser::FromBytes;
use structs::Message;
use tokio::net::UdpSocket;

use crate::structs::{Class, Type, RR};

mod errors;
mod parser;
mod structs;
mod worker;

const MAX_DATAGRAM_SIZE: usize = 40_96;

async fn create_query(message: Message) -> Message {
    let mut response = message.clone();
    let ip = String::from("93.184.216.34");
    let rr = RR {
        name: vec![String::from("example"), String::from("org")],
        _type: Type::A,
        class: Class::IN,
        ttl: 4096,
        rdlength: ip.len() as u16,
        rdata: vec![1, 2, 3, 4],
    };

    response.header.flags |= 0b1000010110000000;
    response.header.ancount = 1;
    response.header.arcount = 0;
    response.answer = Some(rr);

    response
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let local_addr: SocketAddr = "127.0.0.1:8080".parse()?;

    let socket_shared = Arc::new(UdpSocket::bind(local_addr).await?);

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
