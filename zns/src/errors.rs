use thiserror::Error;

use crate::structs::{Type, RCODE};

#[derive(Error, Debug)]
pub enum ZNSError {
    #[error("Parse Error for {object:?}: {message:?}")]
    Parse { object: String, message: String },
    #[error("Reader Error: {message:?}")]
    Reader { message: String },
    #[error("Key Error: {message:?}")]
    Key { message: String },
    #[error("Server error: {message:?}")]
    Servfail { message: String },
    #[error("DNS Query Format Error: {message:?}")]
    Formerr { message: String },
    #[error("Domain name does not exist: {qtype:?} {domain:?}")]
    NXDomain { domain: String, qtype: Type },
    #[error("NotImplemented Error for {object:?}: {message:?}")]
    NotImp { object: String, message: String },
    #[error("Server not authoritative for zone: {message:?}")]
    NotAuth { message: String },
    #[error("I refuse to answer the query: {message:?}")]
    Refused { message: String },
    #[error("Update RR is outside zone: {message:?}")]
    UpdateZone { message: String },
}

impl ZNSError {
    pub fn rcode(&self) -> RCODE {
        match self {
            ZNSError::Formerr { .. } | ZNSError::Parse { .. } | ZNSError::Reader { .. } => {
                RCODE::FORMERR
            }
            ZNSError::Servfail { .. } => RCODE::SERVFAIL,
            ZNSError::NotAuth { .. } => RCODE::NOTAUTH,
            ZNSError::NXDomain { .. } => RCODE::NXDOMAIN,
            ZNSError::NotImp { .. } => RCODE::NOTIMP,
            ZNSError::Refused { .. } | ZNSError::Key { .. } => RCODE::REFUSED,
            ZNSError::UpdateZone { .. } => RCODE::NOTZONE,
        }
    }
}
