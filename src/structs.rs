use int_enum::IntEnum;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Type(RRType),
    Other(u16),
}

#[repr(u16)]
#[derive(Debug, Clone, PartialEq, IntEnum)]
pub enum RRType {
    A = 1,
    SOA = 6,
    KEY = 24, //TODO: change to SIG
    DNSKEY = 48,
    OPT = 41,
    ANY = 255,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Class {
    Class(RRClass),
    Other(u16),
}

#[repr(u16)]
#[derive(Debug, Clone, PartialEq, IntEnum)]
pub enum RRClass {
    IN = 1,
    NONE = 254,
    ANY = 255,
}

#[repr(u16)]
#[allow(dead_code)]
#[derive(Debug, IntEnum, PartialEq)]
pub enum RCODE {
    NOERROR = 0,
    FORMERR = 1,
    SERVFAIL = 2,
    NXDOMAIN = 3,
    NOTIMP = 4,
    REFUSED = 5,
    YXDOMAIN = 6,
    YXRRSET = 7,
    NXRRSET = 8,
    NOTAUTH = 9,
    NOTZONE = 10,
}

pub enum Opcode {
    QUERY = 0,
    UPDATE = 5,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Question {
    pub qname: LabelString,
    pub qtype: Type,   // NOTE: should be QTYPE, right now not really needed
    pub qclass: Class, //NOTE: should be QCLASS, right now not really needed
}

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    pub id: u16,
    pub flags: u16, // |QR|   Opcode  |AA|TC|RD|RA|   Z    |   RCODE   | ; 1 | 4 | 1 | 1 | 1 | 1 | 3 | 4
    pub qdcount: u16,
    pub ancount: u16,
    pub nscount: u16,
    pub arcount: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub header: Header,
    pub question: Vec<Question>,
    pub answer: Vec<RR>,
    pub authority: Vec<RR>,
    pub additional: Vec<RR>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RR {
    pub name: LabelString,
    pub _type: Type,
    pub class: Class,
    pub ttl: i32,
    pub rdlength: u16,
    pub rdata: Vec<u8>,
}

pub type LabelString = Vec<String>;
