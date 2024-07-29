use std::mem::size_of;

use crate::{
    errors::ZNSError,
    reader::Reader,
    structs::{Class, Header, LabelString, Message, Opcode, Question, RRClass, RRType, Type, RR},
};

type Result<T> = std::result::Result<T, ZNSError>;

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
        match RRType::try_from(value) {
            Ok(rrtype) => Type::Type(rrtype),
            Err(x) => Type::Other(x),
        }
    }
}

impl From<u16> for Class {
    fn from(value: u16) -> Self {
        match RRClass::try_from(value) {
            Ok(rrclass) => Class::Class(rrclass),
            Err(x) => Class::Other(x),
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
            Err(ZNSError::Parse {
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
                    ZNSError::Parse {
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
        let qname = LabelString::from_bytes(reader)?;

        if reader.unread_bytes() < 4 {
            Err(ZNSError::Parse {
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
        let _type = Type::from(reader.read_u16()?);
        let class = Class::from(reader.read_u16()?);
        let ttl = reader.read_i32()?;
        let rdlength = reader.read_u16()?;
        if reader.unread_bytes() < rdlength as usize {
            Err(ZNSError::Parse {
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

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn get_rr() -> RR {
        RR {
            name: vec![String::from("example"), String::from("org")],
            _type: Type::Type(RRType::A),
            class: Class::Class(RRClass::IN),
            ttl: 10,
            rdlength: 4,
            rdata: vec![1, 2, 3, 4],
        }
    }

    pub fn get_message() -> Message {
        Message {
            header: Header {
                id: 1,
                flags: 288,
                qdcount: 2,
                ancount: 1,
                nscount: 1,
                arcount: 1,
            },
            question: vec![
                Question {
                    qname: vec![String::from("example"), String::from("org")],
                    qtype: Type::Type(RRType::A),
                    qclass: Class::Class(RRClass::IN),
                },
                Question {
                    qname: vec![String::from("example"), String::from("org")],
                    qtype: Type::Type(RRType::A),
                    qclass: Class::Class(RRClass::IN),
                },
            ],
            answer: vec![get_rr()],
            authority: vec![get_rr()],
            additional: vec![get_rr()],
        }
    }

    #[test]
    fn test_parse_header() {
        let header = Header {
            id: 1,
            flags: 288,
            qdcount: 1,
            ancount: 0,
            nscount: 0,
            arcount: 0,
        };

        let bytes = Header::to_bytes(header.clone());
        let parsed = Header::from_bytes(&mut Reader::new(&bytes));
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), header);
    }

    #[test]
    fn test_parse_question() {
        let question = Question {
            qname: vec![String::from("example"), String::from("org")],
            qtype: Type::Type(RRType::A),
            qclass: Class::Class(RRClass::IN),
        };

        let bytes = Question::to_bytes(question.clone());
        let parsed = Question::from_bytes(&mut Reader::new(&bytes));
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), question);
    }

    #[test]
    fn test_parse_rr() {
        let rr = get_rr();

        let bytes = RR::to_bytes(rr.clone());
        let parsed = RR::from_bytes(&mut Reader::new(&bytes));
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), rr);
    }

    #[test]
    fn test_labelstring() {
        let labelstring = vec![String::from("example"), String::from("org")];

        let bytes = LabelString::to_bytes(labelstring.clone());
        let parsed = LabelString::from_bytes(&mut Reader::new(&bytes));
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), labelstring);
    }

    #[test]
    fn test_labelstring_ptr() {
        let labelstring = vec![String::from("example"), String::from("org")];

        let mut bytes = LabelString::to_bytes(labelstring.clone());

        bytes.insert(0, 0);
        bytes.insert(0, 0);

        let to_read = bytes.len();

        bytes.push(0b11000000);
        bytes.push(0b00000010);

        let mut reader = Reader::new(&bytes);
        let _ = reader.read(to_read);

        let parsed = LabelString::from_bytes(&mut reader);
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), labelstring);
    }

    #[test]
    fn test_labelstring_invalid_ptr() {
        let labelstring = vec![String::from("example"), String::from("org")];

        let mut bytes = LabelString::to_bytes(labelstring.clone());

        bytes.insert(0, 0);
        bytes.insert(0, 0);

        let to_read = bytes.len();

        bytes.push(0b11000000);
        // Not allowed to point to itself or in the future
        bytes.push(to_read as u8);

        let mut reader = Reader::new(&bytes);
        let _ = reader.read(to_read);

        let parsed = LabelString::from_bytes(&mut reader);
        assert!(parsed.is_err());
    }

    #[test]
    fn test_parse_message() {
        let message = get_message();
        let bytes = Message::to_bytes(message.clone());
        let parsed = Message::from_bytes(&mut Reader::new(&bytes));
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), message);
    }
}
