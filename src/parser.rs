use std::mem::size_of;

use crate::{
    errors::ParseError,
    structs::{Class, Header, Message, Question, Type},
};

type Result<T> = std::result::Result<T, ParseError>;

impl TryFrom<u16> for Type {
    type Error = (); //TODO: user better error

    fn try_from(value: u16) -> std::result::Result<Self, ()> {
        match value {
            x if x == Type::A as u16 => Ok(Type::A),
            _ => Err(()),
        }
    }
}

impl TryFrom<u16> for Class {
    type Error = (); //TODO: user better error

    fn try_from(value: u16) -> std::result::Result<Self, ()> {
        match value {
            x if x == Class::IN as u16 => Ok(Class::IN),
            _ => Err(()),
        }
    }
}

// TODO: use Error instead of Option
pub trait FromBytes {
    fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized;
}

impl FromBytes for Header {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != size_of::<Header>() {
            Err(ParseError {
                object: String::from("Header"),
                message: String::from("Size of Header does not match"),
            })
        } else {
            Ok(Header {
                id: u16::from_be_bytes(bytes[0..2].try_into().unwrap()),
                flags: u16::from_be_bytes(bytes[2..4].try_into().unwrap()),
                qdcount: u16::from_be_bytes(bytes[4..6].try_into().unwrap()),
                ancount: u16::from_be_bytes(bytes[6..8].try_into().unwrap()),
                nscount: u16::from_be_bytes(bytes[8..10].try_into().unwrap()),
                arcount: u16::from_be_bytes(bytes[10..12].try_into().unwrap()),
            })
        }
    }
}

//HACK: lots of unsafe unwrap
impl FromBytes for Question {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        // 16 for length octet +  zero length octet
        if bytes.len() < 2 + size_of::<Class>() + size_of::<Type>() {
            Err(ParseError {
                object: String::from("Question"),
                message: String::from("len of bytes smaller then minimum size"),
            })
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
                Err(ParseError {
                    object: String::from("Question"),
                    message: String::from("len of rest bytes smaller then minimum size"),
                })
            } else {
                //Try Parse qtype
                let qtype = Type::try_from(u16::from_be_bytes(bytes[i..i + 2].try_into().unwrap()))
                    .map_err(|_| ParseError {
                        object: String::from("Type"),
                        message: String::from("invalid"),
                    })?;

                //Try Parse qclass
                let qclass =
                    Class::try_from(u16::from_be_bytes(bytes[i + 2..i + 4].try_into().unwrap()))
                        .map_err(|_| ParseError {
                            object: String::from("Class"),
                            message: String::from("invalid"),
                        })?;

                Ok(Question {
                    qname,
                    qtype,
                    qclass,
                })
            }
        }
    }
}

impl FromBytes for Message {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let header = Header::from_bytes(&bytes[0..12])?;
        let question = Question::from_bytes(&bytes[12..])?;
        Ok(Message {
            header,
            question,
            answer: None,
            authority: None,
            additional: None,
        })
    }
}
