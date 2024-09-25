#![cfg(feature = "test-utils")]
use crate::structs::*;

#[cfg(feature = "test-utils")]
use crate::labelstring::LabelString;
pub fn get_rr(name: Option<LabelString>) -> RR {
    RR {
        name: name.unwrap_or(LabelString::from("example.org")),
        _type: Type::Type(RRType::A),
        class: Class::Class(RRClass::IN),
        ttl: 10,
        rdlength: 4,
        rdata: vec![1, 2, 3, 4],
    }
}

pub fn get_message(name: Option<LabelString>) -> Message {
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
                qname: name.clone().unwrap_or(LabelString::from("example.org")),
                qtype: Type::Type(RRType::A),
                qclass: Class::Class(RRClass::IN),
            },
            Question {
                qname: name.clone().unwrap_or(LabelString::from("example.org")),
                qtype: Type::Type(RRType::A),
                qclass: Class::Class(RRClass::IN),
            },
        ],
        answer: vec![get_rr(name.clone())],
        authority: vec![get_rr(name.clone())],
        additional: vec![get_rr(name)],
    }
}
