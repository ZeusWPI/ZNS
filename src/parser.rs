use std::{mem::size_of, vec};

use crate::{
    errors::ParseError,
    reader::Reader,
    structs::{
        Class, Header, KeyRData, LabelString, Message, Opcode, Question, RRClass, RRType, Type, RR,
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
        //TODO: use macro
        match value {
            x if x == RRType::A as u16 => Type::Type(RRType::A),
            x if x == RRType::OPT as u16 => Type::Type(RRType::OPT),
            x if x == RRType::SOA as u16 => Type::Type(RRType::SOA),
            x if x == RRType::ANY as u16 => Type::Type(RRType::SOA),
            x if x == RRType::KEY as u16 => Type::Type(RRType::KEY),
            x if x == RRType::DNSKEY as u16 => Type::Type(RRType::DNSKEY),
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
    fn from_bytes(reader: &mut Reader) -> Result<Self>
    where
        Self: Sized;
}

pub trait ToBytes {
    fn to_bytes(s: Self) -> Vec<u8>
    where
        Self: Sized;
}

impl FromBytes for Header {
    fn from_bytes(reader: &mut Reader) -> Result<Self> {
        if reader.unread_bytes() < size_of::<Header>() {
            Err(ParseError {
                object: String::from("Header"),
                message: String::from("Size of Header does not match"),
            })
        } else {
            Ok(Header {
                id: reader.read_u16()?,
                flags: reader.read_u16()?,
                qdcount: reader.read_u16()?,
                ancount: reader.read_u16()?,
                nscount: reader.read_u16()?,
                arcount: reader.read_u16()?,
            })
        }
    }
}

impl ToBytes for Header {
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
    fn from_bytes(reader: &mut Reader) -> Result<Self> {
        let mut out = vec![];

        // Parse qname labels
        let mut code = reader.read_u8()?;
        while code != 0 && (code & 0b11000000 == 0) && reader.unread_bytes() > code as usize {
            out.push(
                String::from_utf8(reader.read(code as usize)?.to_vec()).map_err(|e| {
                    ParseError {
                        object: String::from("Label"),
                        message: e.to_string(),
                    }
                })?,
            );
            code = reader.read_u8()?;
        }

        if code & 0b11000000 != 0 {
            let offset = (((code & 0b00111111) as u16) << 8) | reader.read_u8()? as u16;
            let mut reader_past = reader.seek(offset as usize)?;
            out.extend(LabelString::from_bytes(&mut reader_past)?);
        }

        Ok(out)
    }
}

impl ToBytes for LabelString {
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
    fn from_bytes(reader: &mut Reader) -> Result<Self> {
        // 16 for length octet +  zero length octet
        if reader.unread_bytes() < 2 + size_of::<Class>() + size_of::<Type>() {
            Err(ParseError {
                object: String::from("Question"),
                message: String::from("len of bytes smaller then minimum size"),
            })
        } else {
            let qname = LabelString::from_bytes(reader)?;

            if reader.unread_bytes() < 4 {
                Err(ParseError {
                    object: String::from("Question"),
                    message: String::from("len of rest bytes smaller then minimum size"),
                })
            } else {
                //Try Parse qtype
                let qtype = Type::from(reader.read_u16()?);

                //Try Parse qclass
                let qclass = Class::from(reader.read_u16()?);

                Ok(Question {
                    qname,
                    qtype,
                    qclass,
                })
            }
        }
    }
}

impl ToBytes for Question {
    fn to_bytes(question: Self) -> Vec<u8> {
        let mut result = LabelString::to_bytes(question.qname);
        result.extend(u16::to_be_bytes(question.qtype.into()));
        result.extend(u16::to_be_bytes(question.qclass.into()));
        result
    }
}

impl FromBytes for RR {
    fn from_bytes(reader: &mut Reader) -> Result<Self> {
        let name = LabelString::from_bytes(reader)?;
        if reader.unread_bytes() < size_of::<Type>() + size_of::<Class>() + 6 {
            Err(ParseError {
                object: String::from("RR"),
                message: String::from("len of rest of bytes smaller then minimum size"),
            })
        } else {
            let _type = Type::from(reader.read_u16()?);
            let class = Class::from(reader.read_u16()?);
            let ttl = reader.read_i32()?;
            let rdlength = reader.read_u16()?;
            if reader.unread_bytes() < rdlength as usize {
                Err(ParseError {
                    object: String::from("RR"),
                    message: String::from("len of rest of bytes not equal to rdlength"),
                })
            } else {
                Ok(RR {
                    name,
                    _type,
                    class,
                    ttl,
                    rdlength,
                    rdata: reader.read(rdlength as usize)?,
                })
            }
        }
    }
}

impl ToBytes for RR {
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
    fn from_bytes(reader: &mut Reader) -> Result<Self> {
        let header = Header::from_bytes(reader)?;

        let mut question = vec![];
        for _ in 0..header.qdcount {
            question.push(Question::from_bytes(reader)?);
        }

        let mut answer = vec![];
        for _ in 0..header.ancount {
            answer.push(RR::from_bytes(reader)?);
        }

        let mut authority = vec![];
        for _ in 0..header.nscount {
            authority.push(RR::from_bytes(reader)?);
        }

        let mut additional = vec![];
        for _ in 0..header.arcount {
            additional.push(RR::from_bytes(reader)?);
        }

        Ok(Message {
            header,
            question,
            answer,
            authority,
            additional,
        })
    }
}

impl ToBytes for Message {
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
    fn from_bytes(reader: &mut Reader) -> Result<Self> {
        if reader.unread_bytes() < 18 {
            Err(ParseError {
                object: String::from("KeyRData"),
                message: String::from("invalid rdata"),
            })
        } else {
            Ok(KeyRData {
                type_covered: reader.read_u16()?,
                algo: reader.read_u8()?,
                labels: reader.read_u8()?,
                original_ttl: reader.read_u32()?,
                signature_expiration: reader.read_u32()?,
                signature_inception: reader.read_u32()?,
                key_tag: reader.read_u16()?,
                signer: LabelString::from_bytes(reader)?,
                signature: reader.read(reader.unread_bytes())?,
            })
        }
    }
}
