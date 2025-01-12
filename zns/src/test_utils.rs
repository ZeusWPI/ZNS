#![cfg(feature = "test-utils")]
use rand::distributions::Alphanumeric;
use rand::Rng;

use crate::structs::*;

use crate::labelstring::LabelString;
pub fn get_rr(name: Option<LabelString>) -> RR {
    RR {
        name: name.unwrap_or(LabelString::from("example.org")),
        _type: Type::Type(RRType::A),
        class: Class::Class(RRClass::IN),
        ttl: 10,
        rdlength: 4,
        rdata: RData::Vec(vec![
            (rand::random::<u8>()),
            (rand::random::<u8>()),
            (rand::random::<u8>()),
            (rand::random::<u8>()),
        ]),
    }
}

pub fn get_cname_rr(name: Option<LabelString>) -> RR {
    RR {
        name: name.unwrap_or(LabelString::from("example.org")),
        _type: Type::Type(RRType::CNAME),
        class: Class::Class(RRClass::IN),
        ttl: 10,
        rdlength: 4,
        rdata: RData::LabelString(random_domain()),
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

fn random_domain() -> LabelString {
    let part1 = random_string();
    let part2 = random_string();
    LabelString::from(&format!("{part1}.{part2}"))
}

fn random_string() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect()
}
