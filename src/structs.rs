use serde::Deserialize;

#[repr(u16)]
#[derive(Debug, Clone, Deserialize)]
pub enum Type {
    A = 1,
}

#[repr(u16)]
#[derive(Debug, Clone)]
pub enum Class {
    IN = 1,
}

#[derive(Debug, Clone)]
pub struct Question {
    pub qname: Vec<String>,
    pub qtype: Type,   // NOTE: should be QTYPE, right now not really needed
    pub qclass: Class, //NOTE: should be QCLASS, right now not really needed
}

#[derive(Debug, Clone)]
pub struct Header {
    pub id: u16,
    pub flags: u16, // |QR|   Opcode  |AA|TC|RD|RA|   Z    |   RCODE   | ; 1 | 4 | 1 | 1 | 1 | 1 | 3 | 4
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub header: Header,
    pub question: Question,
    pub answer: Option<RR>,
    pub authority: Option<RR>,
    pub additional: Option<RR>,
}

#[derive(Debug, Clone)]
pub struct RR {
    pub name: Vec<String>,
    pub _type: Type,
    pub class: Class,
    pub ttl: i32,
    pub rdlength: u16,
    pub rdata: Vec<u8>,
}

pub type LabelString = (Vec<String>, usize);

#[derive(Debug)]
pub struct Response {
    field: Type,
}
