use std::{error::Error, net::SocketAddr};

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
        name: vec![String::from("example"),String::from("org")],
        _type: Type::A,
        class: Class::IN,
        ttl: 4096,
        rdlength: ip.len() as u16,
        rdata: ip,
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

    let socket = UdpSocket::bind(local_addr).await?;

    let mut data = vec![0u8; MAX_DATAGRAM_SIZE];
    let (len,addr) = socket.recv_from(&mut data).await?;
    match Message::from_bytes(&data[..len]) {
        Ok(message) => {
            tokio::spawn(async move {
                let response = create_query(message).await;
                println!("{:?}",response);
                let vec = Message::to_bytes(&response);
                let decoded = Message::from_bytes(vec.as_slice());
                println!("{:?}",decoded);
                let _ = socket.send_to(vec.as_slice(),addr).await;
            });
        }
        Err(err) => println!("{}", err),
    };
    loop {}
}
