use std::{mem::size_of, vec};

use crate::{
    errors::ParseError,
    structs::{
        Class, Header, KeyRData, LabelString, Message, Opcode, Question, RRClass, RRType,
        Type, RR,
    },
};

type Result<T> = std::result::Result<T, ParseError>;

impl From<Type> for u16 {
    fn from(value: Type) -> Self {
        match value {
            Type::Type(t) => t as u16,
            Type::Other(x) => x,
        }
    }
}

impl From<Type> for i32 {
    fn from(value: Type) -> Self {
        Into::<u16>::into(value) as i32
    }
}

impl From<Class> for i32 {
    fn from(value: Class) -> Self {
        Into::<u16>::into(value) as i32
    }
}

impl From<Class> for u16 {
    fn from(value: Class) -> Self {
        match value {
            Class::Class(t) => t as u16,
            Class::Other(x) => x,
        }
    }
}

impl From<u16> for Type {
    fn from(value: u16) -> Self {
        match value {
            x if x == RRType::A as u16 => Type::Type(RRType::A),
            x if x == RRType::OPT as u16 => Type::Type(RRType::OPT),
            x if x == RRType::SOA as u16 => Type::Type(RRType::SOA),
            x if x == RRType::ANY as u16 => Type::Type(RRType::SOA),
            x if x == RRType::KEY as u16 => Type::Type(RRType::KEY),
            x => Type::Other(x),
        }
    }
}

impl From<u16> for Class {
    fn from(value: u16) -> Self {
        match value {
            x if x == RRClass::IN as u16 => Class::Class(RRClass::IN),
            x if x == RRClass::ANY as u16 => Class::Class(RRClass::ANY),
            x if x == RRClass::NONE as u16 => Class::Class(RRClass::NONE),
            x => Class::Other(x),
        }
    }
}

impl TryFrom<u16> for Opcode {
    type Error = String;

    fn try_from(value: u16) -> std::result::Result<Self, String> {
        match value {
            x if x == Opcode::QUERY as u16 => Ok(Opcode::QUERY),
            x if x == Opcode::UPDATE as u16 => Ok(Opcode::UPDATE),
            _ => Err(format!("Invalid Opcode value: {}", value)),
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

impl Type {
    pub fn to_data(&self, text: &String) -> Result<Vec<u8>> {
        match self {
            Type::Type(RRType::A) => {
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
            _ => todo!(),
        }
    }
    pub fn from_data(&self, bytes: &[u8]) -> Result<String> {
        match self {
            Type::Type(RRType::A) => {
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
            _ => todo!(),
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
        let mut out = vec![];

        // Parse qname labels
        while bytes[*i] != 0
            && (bytes[*i] & 0b11000000 == 0)
            && bytes[*i] as usize + *i < bytes.len()
        {
            out.push(
                String::from_utf8(bytes[*i + 1..bytes[*i] as usize + 1 + *i].to_vec()).unwrap(),
            );
            *i += bytes[*i] as usize + 1;
        }

        if bytes[*i] & 0b11000000 != 0 {
            println!("YOOW");
            let offset = u16::from_be_bytes(bytes[*i..*i + 2].try_into().unwrap()) & 0b0011111111111111;
            if *i <= offset as usize {
                return Err(ParseError {
                    object: String::from("Label"),
                    message: String::from("Invalid PTR"),
                });
            } else {
                out.extend(LabelString::from_bytes(bytes, &mut (offset as usize))?);
                *i += 1;
            }
        }

        *i += 1;
        Ok(out)
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
                let qtype = Type::from(u16::from_be_bytes(bytes[*i..*i + 2].try_into().unwrap()));

                //Try Parse qclass
                let qclass = Class::from(u16::from_be_bytes(
                    bytes[*i + 2..*i + 4].try_into().unwrap(),
                ));

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
        result.extend(u16::to_be_bytes(question.qtype.into()));
        result.extend(u16::to_be_bytes(question.qclass.into()));
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
            let _type = Type::from(u16::from_be_bytes(bytes[*i..*i + 2].try_into().unwrap()));

            let class = Class::from(u16::from_be_bytes(
                bytes[*i + 2..*i + 4].try_into().unwrap(),
            ));

            let ttl = i32::from_be_bytes(bytes[*i + 4..*i + 8].try_into().unwrap());
            let rdlength = u16::from_be_bytes(bytes[*i + 8..*i + 10].try_into().unwrap());

            if bytes.len() - *i - 10 < rdlength as usize {
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
        result.extend(u16::to_be_bytes(rr._type.into()));
        result.extend(u16::to_be_bytes(rr.class.into()));
        result.extend(i32::to_be_bytes(rr.ttl.to_owned()));
        result.extend(u16::to_be_bytes(rr.rdata.len() as u16));
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
        println!("{:#?}", question);
        println!("{:#?}", header);

        let mut answer = vec![];
        for _ in 0..header.ancount {
            answer.push(RR::from_bytes(&bytes, i)?);
        }

        let mut authority = vec![];
        for _ in 0..header.nscount {
            authority.push(RR::from_bytes(&bytes, i)?);
        }
        println!("{:#?}", authority);

        let mut additional = vec![];
        for _ in 0..header.arcount {
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

impl FromBytes for KeyRData {
    fn from_bytes(bytes: &[u8], i: &mut usize) -> Result<Self> {
        if bytes.len() < 18 {
            Err(ParseError {
                object: String::from("KeyRData"),
                message: String::from("invalid rdata"),
            })
        } else {
            *i = 18;
            Ok(KeyRData {
                type_covered: u16::from_be_bytes(bytes[0..2].try_into().unwrap()),
                algo: bytes[2],
                labels: bytes[3],
                original_ttl: u32::from_be_bytes(bytes[4..8].try_into().unwrap()),
                signature_expiration: u32::from_be_bytes(bytes[8..12].try_into().unwrap()),
                signature_inception: u32::from_be_bytes(bytes[12..16].try_into().unwrap()),
                key_tag: u16::from_be_bytes(bytes[16..18].try_into().unwrap()),
                signer: LabelString::from_bytes(bytes, i)?,
                signature: bytes[*i..bytes.len()].to_vec(),
            })
        }
    }

    fn to_bytes(s: Self) -> Vec<u8>
    where
        Self: Sized,
    {
        todo!()
    }
}
