use std::{error::Error, net::SocketAddr};

use parser::FromBytes;
use structs::Message;
use tokio::net::UdpSocket;

mod errors;
mod parser;
mod structs;
mod worker;

const MAX_DATAGRAM_SIZE: usize = 40_96;

async fn create_query(message: Message) {
    println!("{:?}", message);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let local_addr: SocketAddr = "127.0.0.1:8080".parse()?;

    let socket = UdpSocket::bind(local_addr).await?;

    let mut data = vec![0u8; MAX_DATAGRAM_SIZE];
    loop {
        let len = socket.recv(&mut data).await?;
        match Message::from_bytes(&data[..len]) {
            Ok(message) => {
                tokio::spawn(async move { create_query(message).await });
            }
            Err(err) => println!("{}", err),
        };
    }
}
