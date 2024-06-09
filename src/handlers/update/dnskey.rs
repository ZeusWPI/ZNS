use base64::prelude::*;

use crate::{errors::ParseError, parser::FromBytes, reader::Reader};

use super::sig::Sig;

/// https://datatracker.ietf.org/doc/html/rfc4034#section-2
#[derive(Debug)]
pub(super) struct DNSKeyRData {
    pub flags: u16,
    pub protocol: u8,
    pub algorithm: u8,
    pub public_key: Vec<u8>,
}

//TODO: validate values
impl FromBytes for DNSKeyRData {
    fn from_bytes(reader: &mut Reader) -> Result<Self, ParseError> {
        Ok(DNSKeyRData {
            flags: reader.read_u16()?,
            protocol: reader.read_u8()?,
            algorithm: reader.read_u8()?,
            public_key: reader.read(reader.unread_bytes())?,
        })
    }
}

impl DNSKeyRData {
    pub fn verify(&self, sig: &Sig) -> bool {
        let encoded = BASE64_STANDARD.encode(&self.public_key);
        match self.algorithm {
            15 => sig.verify_ed25519(&encoded),
            _ => false,
        }
    }
}
