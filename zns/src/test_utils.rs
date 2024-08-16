#![cfg(feature = "test-utils")]
use crate::structs::*;

#[cfg(feature = "test-utils")]
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
