use std::{error::Error, mem::size_of, net::SocketAddr};

use tokio::net::UdpSocket;

mod worker;

#[repr(u16)]
#[derive(Debug)]
enum Type {
    A = 1,
}

#[repr(u16)]
#[derive(Debug)]
enum Class {
    IN = 1,
}

#[derive(Debug)]
struct Question {
    qname: Vec<String>, // TODO: not padded
    qtype: Type,        // NOTE: should be QTYPE, right now not really needed
    qclass: Class,
}

#[derive(Debug)]
struct Header {
    id: u16,
    flags: u16, // |QR|   Opcode  |AA|TC|RD|RA|   Z    |   RCODE   | ; 1 | 4 | 1 | 1 | 1 | 1 | 3 | 4
    qdcount: u16,
    ancount: u16,
    nscount: u16,
    arcount: u16,
}

#[derive(Debug)]
pub struct Message {
    header: Option<Header>,
    question: Option<Question>,
    answer: Option<RR>,
    authority: Option<RR>,
    additional: Option<RR>,
}

#[derive(Debug)]
struct RR {
    name: String,
    t: u16,
    class: u16,
    ttl: u32,
    rdlength: u16,
    rdata: String,
}

#[derive(Debug)]
struct Response {
    field: Type,
}

const MAX_DATAGRAM_SIZE: usize = 40_96;

impl TryFrom<u16> for Type {
    type Error = (); //TODO: user better error

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            x if x == Type::A as u16 => Ok(Type::A),
            _ => Err(()),
        }
    }
}

impl TryFrom<u16> for Class {
    type Error = (); //TODO: user better error

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            x if x == Class::IN as u16 => Ok(Class::IN),
            _ => Err(()),
        }
    }
}

// TODO: use Error instead of Option
trait FromBytes {
    fn from_bytes(bytes: &[u8]) -> Option<Self>
    where
        Self: Sized;
}

impl FromBytes for Header {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != size_of::<Header>() {
            return None; // Size of header should match
        }

        Some(Header {
            id: u16::from_be_bytes(bytes[0..2].try_into().unwrap()),
            flags: u16::from_be_bytes(bytes[2..4].try_into().unwrap()),
            qdcount: u16::from_be_bytes(bytes[4..6].try_into().unwrap()),
            ancount: u16::from_be_bytes(bytes[6..8].try_into().unwrap()),
            nscount: u16::from_be_bytes(bytes[8..10].try_into().unwrap()),
            arcount: u16::from_be_bytes(bytes[10..12].try_into().unwrap()),
        })
    }
}

//HACK: lots of unsafe unwrap
impl FromBytes for Question {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        // 16 for length octet +  zero length octet
        if bytes.len() < 2 + size_of::<Class>() + size_of::<Type>() {
            None
        } else {
            let mut qname = vec![];
            let mut i = 0;

            // Parse qname labels
            while bytes[i] != 0 && bytes[i] as usize + i < bytes.len() {
                qname.push(
                    String::from_utf8(bytes[i + 1..bytes[i] as usize + 1 + i].to_vec()).unwrap(),
                );
                i += bytes[i] as usize + 1;
            }
            i += 1;

            if bytes.len() - i < size_of::<Class>() + size_of::<Type>() {
                None
            } else {
                //Try Parse qtype
                let qtype = Type::try_from(u16::from_be_bytes(bytes[i..i + 2].try_into().unwrap()))
                    .unwrap();

                //Try Parse qclass
                let qclass =
                    Class::try_from(u16::from_be_bytes(bytes[i + 2..i + 4].try_into().unwrap()))
                        .unwrap();

                Some(Question {
                    qname,
                    qtype,
                    qclass,
                })
            }
        }
    }
}

impl FromBytes for Message {
    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let header = Header::from_bytes(&bytes[0..12]);
        let question = Question::from_bytes(&bytes[12..]);
        if header.is_some() {
            Some(Message {
                header,
                question,
                answer: None,
                authority: None,
                additional: None,
            })
        } else {
            None
        }
    }
}

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
        let message = Message::from_bytes(&data[..len]);
        if message.is_some() {
            tokio::spawn(async move {
                create_query(message.unwrap()).await;
            });
        }
    }
}
