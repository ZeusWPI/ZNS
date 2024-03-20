use std::{mem::size_of, vec};

use crate::{
    errors::ParseError,
    structs::{Class, Header, LabelString, Message, OptRR, Question, Type, RR},
};

type Result<T> = std::result::Result<T, ParseError>;

impl TryFrom<u16> for Type {
    type Error = String;

    fn try_from(value: u16) -> std::result::Result<Self, String> {
        match value {
            x if x == Type::A as u16 => Ok(Type::A),
            _ => Err(format!("Invalid Type value: {}", value)),
        }
    }
}

impl TryFrom<u16> for Class {
    type Error = String;

    fn try_from(value: u16) -> std::result::Result<Self, String> {
        match value {
            x if x == Class::IN as u16 => Ok(Class::IN),
            _ => Err(format!("Invalid Class value: {}", value)),
        }
    }
}

pub trait FromBytes {
    fn from_bytes(bytes: &[u8], i: &mut usize) -> Result<Self>
    where
        Self: Sized;
    fn to_bytes(s: Self) -> Vec<u8>
    where
        Self: Sized;
}

pub fn parse_opt_type(bytes: &[u8]) -> Result<Vec<OptRR>> {
    let mut pairs: Vec<OptRR> = vec![];
    let mut i: usize = 0;
    while i + 4 <= bytes.len() {
        let length = u16::from_be_bytes(bytes[i + 2..i + 4].try_into().unwrap());
        pairs.push(OptRR {
            code: u16::from_be_bytes(bytes[i..i + 2].try_into().unwrap()),
            length,
            rdata: bytes[i + 4..i + 4 + length as usize]
                .try_into()
                .map_err(|_| ParseError {
                    object: String::from("Type::OPT"),
                    message: String::from("Invalid OPT DATA"),
                })?,
        });
        i += 4 + length as usize;
    }

    Ok(pairs)
}

impl Type {
    pub fn to_data(&self, text: &String) -> Result<Vec<u8>> {
        match self {
            Type::A => {
                let arr: Vec<u8> = text
                    .split(".")
                    .filter_map(|s| s.parse::<u8>().ok())
                    .collect();
                if arr.len() == 4 {
                    Ok(arr)
                } else {
                    Err(ParseError {
                        object: String::from("Type::A"),
                        message: String::from("Invalid IPv4 address"),
                    })
                }
            }
            Type::SOA => todo!(),
        }
    }
    pub fn from_data(&self, bytes: &[u8]) -> Result<String> {
        match self {
            Type::A => {
                if bytes.len() == 4 {
                    let arr: Vec<String> = bytes.iter().map(|b| b.to_string()).collect();
                    Ok(arr.join("."))
                } else {
                    Err(ParseError {
                        object: String::from("Type::A"),
                        message: String::from("Invalid Ipv4 address bytes"),
                    })
                }
            }
            Type::SOA => todo!(),
        }
    }
}

impl FromBytes for Header {
    fn from_bytes(bytes: &[u8], i: &mut usize) -> Result<Self> {
        if bytes.len() < size_of::<Header>() {
            Err(ParseError {
                object: String::from("Header"),
                message: String::from("Size of Header does not match"),
            })
        } else {
            *i += size_of::<Header>();
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

    fn to_bytes(header: Self) -> Vec<u8> {
        let mut result: [u8; size_of::<Header>()] = [0; size_of::<Header>()];

        result[0..2].copy_from_slice(&u16::to_be_bytes(header.id));
        result[2..4].copy_from_slice(&u16::to_be_bytes(header.flags));
        result[4..6].copy_from_slice(&u16::to_be_bytes(header.qdcount));
        result[6..8].copy_from_slice(&u16::to_be_bytes(header.ancount));
        result[8..10].copy_from_slice(&u16::to_be_bytes(header.nscount));
        result[10..12].copy_from_slice(&u16::to_be_bytes(header.arcount));

        result.to_vec()
    }
}

impl FromBytes for LabelString {
    fn from_bytes(bytes: &[u8], i: &mut usize) -> Result<Self> {
        let mut qname = vec![];

        // Parse qname labels
        while bytes[*i] != 0 && bytes[*i] as usize + *i < bytes.len() {
            qname.push(
                String::from_utf8(bytes[*i + 1..bytes[*i] as usize + 1 + *i].to_vec()).unwrap(),
            );
            *i += bytes[*i] as usize + 1;
        }

        *i += 1;
        Ok(qname)
    }

    fn to_bytes(name: Self) -> Vec<u8> {
        let mut result: Vec<u8> = vec![];
        for label in name {
            result.push(label.len() as u8);
            result.extend(label.as_bytes());
        }
        result.push(0);
        result
    }
}

impl FromBytes for Question {
    fn from_bytes(bytes: &[u8], i: &mut usize) -> Result<Self> {
        // 16 for length octet +  zero length octet
        if bytes.len() < 2 + size_of::<Class>() + size_of::<Type>() {
            Err(ParseError {
                object: String::from("Question"),
                message: String::from("len of bytes smaller then minimum size"),
            })
        } else {
            let qname = LabelString::from_bytes(bytes, i)?;

            if bytes.len() - *i < size_of::<Class>() + size_of::<Type>() {
                Err(ParseError {
                    object: String::from("Question"),
                    message: String::from("len of rest bytes smaller then minimum size"),
                })
            } else {
                //Try Parse qtype
                let qtype =
                    Type::try_from(u16::from_be_bytes(bytes[*i..*i + 2].try_into().unwrap()))
                        .map_err(|_| ParseError {
                            object: String::from("Type"),
                            message: String::from("invalid"),
                        })?;

                //Try Parse qclass
                let qclass = Class::try_from(u16::from_be_bytes(
                    bytes[*i + 2..*i + 4].try_into().unwrap(),
                ))
                .map_err(|_| ParseError {
                    object: String::from("Class"),
                    message: String::from("invalid"),
                })?;

                *i += 4; // For qtype and qclass => 4 bytes

                Ok(Question {
                    qname,
                    qtype,
                    qclass,
                })
            }
        }
    }

    fn to_bytes(question: Self) -> Vec<u8> {
        let mut result = LabelString::to_bytes(question.qname);
        result.extend(u16::to_be_bytes(question.qtype.to_owned() as u16));
        result.extend(u16::to_be_bytes(question.qclass.to_owned() as u16));
        result
    }
}

impl FromBytes for RR {
    fn from_bytes(bytes: &[u8], i: &mut usize) -> Result<Self> {
        let name = LabelString::from_bytes(bytes, i)?;
        if bytes.len() - *i < size_of::<Type>() + size_of::<Class>() + 6 {
            Err(ParseError {
                object: String::from("RR"),
                message: String::from("len of rest of bytes smaller then minimum size"),
            })
        } else {
            let _type = Type::try_from(u16::from_be_bytes(bytes[*i..*i + 2].try_into().unwrap()))
                .map_err(|_| ParseError {
                object: String::from("Type"),
                message: String::from("invalid"),
            })?;

            let class = Class::try_from(u16::from_be_bytes(
                bytes[*i + 2..*i + 4].try_into().unwrap(),
            ))
            .map_err(|_| ParseError {
                object: String::from("Class"),
                message: String::from("invalid"),
            })?;

            let ttl = i32::from_be_bytes(bytes[*i + 4..*i + 8].try_into().unwrap());
            let rdlength = u16::from_be_bytes(bytes[*i + 8..*i + 10].try_into().unwrap());

            if bytes.len() - *i - 10 != rdlength as usize {
                Err(ParseError {
                    object: String::from("RR"),
                    message: String::from("len of rest of bytes not equal to rdlength"),
                })
            } else {
                *i += 10 + rdlength as usize;
                Ok(RR {
                    name,
                    _type,
                    class,
                    ttl,
                    rdlength,
                    rdata: bytes[*i - rdlength as usize..*i].to_vec(),
                })
            }
        }
    }

    fn to_bytes(rr: Self) -> Vec<u8> {
        let mut result = LabelString::to_bytes(rr.name);
        result.extend(u16::to_be_bytes(rr._type.to_owned() as u16));
        result.extend(u16::to_be_bytes(rr.class.to_owned() as u16));
        result.extend(i32::to_be_bytes(rr.ttl.to_owned()));
        result.extend(u16::to_be_bytes(4 as u16));
        result.extend(rr.rdata);
        result
    }
}

impl FromBytes for Message {
    fn from_bytes(bytes: &[u8], i: &mut usize) -> Result<Self> {
        let header = Header::from_bytes(&bytes, i)?;

        let mut question = vec![];
        for _ in 0..header.qdcount {
            question.push(Question::from_bytes(&bytes, i)?);
        }

        let mut answer = vec![];
        for _ in 0..header.ancount {
            answer.push(RR::from_bytes(&bytes, i)?);
        }

        let mut authority = vec![];
        for _ in 0..header.nscount {
            authority.push(RR::from_bytes(&bytes, i)?);
        }

        let mut additional = vec![];
        for _ in 0..header.nscount {
            additional.push(RR::from_bytes(&bytes, i)?);
        }

        Ok(Message {
            header,
            question,
            answer,
            authority,
            additional,
        })
    }

    fn to_bytes(message: Self) -> Vec<u8> {
        let mut result = vec![];
        result.extend(Header::to_bytes(message.header));

        for question in message.question {
            result.extend(Question::to_bytes(question));
        }
        for answer in message.answer {
            result.extend(RR::to_bytes(answer));
        }
        for auth in message.authority {
            result.extend(RR::to_bytes(auth));
        }
        for additional in message.additional {
            result.extend(RR::to_bytes(additional));
        }
        result
    }
}
