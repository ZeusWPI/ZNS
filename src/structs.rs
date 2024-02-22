
#[repr(u16)]
#[derive(Debug)]
pub enum Type {
    A = 1,
}

#[repr(u16)]
#[derive(Debug)]
pub enum Class {
    IN = 1,
}

#[derive(Debug)]
pub struct Question {
    pub qname: Vec<String>, // TODO: not padded
    pub qtype: Type,        // NOTE: should be QTYPE, right now not really needed
    pub qclass: Class,
}

#[derive(Debug)]
pub struct Header {
    pub id: u16,
    pub flags: u16, // |QR|   Opcode  |AA|TC|RD|RA|   Z    |   RCODE   | ; 1 | 4 | 1 | 1 | 1 | 1 | 3 | 4
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
}

#[derive(Debug)]
pub struct Message {
    pub header: Header,
    pub question: Question,
    pub answer: Option<RR>,
    pub authority: Option<RR>,
    pub additional: Option<RR>,
}

#[derive(Debug)]
pub struct RR {
    name: String,
    t: u16,
    class: u16,
    ttl: u32,
    rdlength: u16,
    rdata: String,
}

#[derive(Debug)]
pub struct Response {
    field: Type,
}
