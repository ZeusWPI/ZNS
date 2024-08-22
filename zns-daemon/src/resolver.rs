use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpSocket, UdpSocket};
use zns::errors::ZNSError;
use zns::parser::{FromBytes, ToBytes};
use zns::reader::Reader;
use zns::structs::{Header, Message, RCODE};

use crate::db::lib::get_connection;
use crate::handlers::{Handler, ResponseHandler};

const MAX_DATAGRAM_SIZE: usize = 512;

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

async fn get_response(bytes: &[u8]) -> Vec<u8> {
    let mut reader = Reader::new(bytes);
    Message::to_bytes(match Message::from_bytes(&mut reader) {
        Ok(mut message) => match Handler::handle(&message, bytes, &mut get_connection()).await {
            Ok(mut response) => {
                response.set_response(RCODE::NOERROR);
                response
            }
            Err(e) => {
                eprintln!("{}", e);
                message.set_response(e.rcode());
                message
            }
        },
        Err(err) => handle_parse_error(bytes, err),
    })
}

pub async fn udp_listener_loop(addr: SocketAddr) -> Result<(), Box<dyn Error>> {
    let socket_shared = Arc::new(UdpSocket::bind(addr).await?);
    loop {
        let mut data = vec![0u8; MAX_DATAGRAM_SIZE];
        let (len, addr) = socket_shared.recv_from(&mut data).await?;
        let socket = socket_shared.clone();
        tokio::spawn(async move {
            let response = get_response(&data[..len]).await;
            // TODO: if length is larger then 512 bytes, message should be truncated
            let _ = socket.send_to(&response, addr).await;
        });
    }
}

pub async fn tcp_listener_loop(addr: SocketAddr) -> Result<(), Box<dyn Error>> {
    let socket = TcpSocket::new_v4()?;
    socket.bind(addr)?;
    let listener = socket.listen(1024)?;
    loop {
        let (mut stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            if stream.readable().await.is_ok() {
                if let Ok(length) = stream.read_u16().await {
                    let mut buf = Vec::with_capacity(length as usize);
                    if stream
                        .try_read_buf(&mut buf)
                        .is_ok_and(|v| v == length as usize)
                    {
                        let response = get_response(&buf).await;
                        if stream.writable().await.is_ok() {
                            let _ = stream.write_u16(response.len() as u16).await;
                            let _ = stream.try_write(&response);
                        }
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use zns::structs::{Class, Question, RRClass, RRType, Type};

    use super::*;

    #[tokio::test]
    async fn test_get_response() {
        let message = Message {
            header: Header {
                id: 1,
                flags: 288,
                qdcount: 1,
                ancount: 0,
                nscount: 0,
                arcount: 0,
            },
            question: vec![Question {
                qname: vec![String::from("example"), String::from("org")],
                qtype: Type::Type(RRType::A),
                qclass: Class::Class(RRClass::IN),
            }],
            answer: vec![],
            authority: vec![],
            additional: vec![],
        };

        let response = get_response(&Message::to_bytes(message)).await;
        let mut reader = Reader::new(&response);

        assert_eq!(
            Message::from_bytes(&mut reader).unwrap().get_rcode(),
            Ok(RCODE::NXDOMAIN)
        );
    }
}
