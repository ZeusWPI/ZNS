use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::UdpSocket;

use crate::errors::ZNSError;
use crate::handlers::{Handler, ResponseHandler};
use crate::parser::{FromBytes, ToBytes};
use crate::reader::Reader;
use crate::structs::{Header, Message, RCODE};

const MAX_DATAGRAM_SIZE: usize = 4096;

fn handle_parse_error(bytes: &[u8], err: ZNSError) -> Message {
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

    let mut message = Message {
        header,
        question: vec![],
        answer: vec![],
        authority: vec![],
        additional: vec![],
    };
    message.set_response(RCODE::FORMERR);
    message
}

async fn get_response(bytes: &[u8]) -> Message {
    let mut reader = Reader::new(bytes);
    match Message::from_bytes(&mut reader) {
        Ok(mut message) => match Handler::handle(&message, bytes).await {
            Ok(mut response) => {
                response.set_response(RCODE::NOERROR);
                response
            }
            Err(e) => {
                eprintln!("{}", e.to_string());
                message.set_response(e.rcode());
                message
            }
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
