use zns::{errors::ZNSError, parser::FromBytes, reader::Reader};

use super::sig::Algorithm;

/// https://datatracker.ietf.org/doc/html/rfc4034#section-2
#[derive(Debug)]
#[allow(dead_code)]
pub struct DNSKeyRData {
    pub flags: u16,
    pub protocol: u8,
    pub algorithm: Algorithm,
    pub public_key: Vec<u8>,
}

//TODO: validate values
impl FromBytes for DNSKeyRData {
    fn from_bytes(reader: &mut Reader) -> Result<Self, ZNSError> {
        Ok(DNSKeyRData {
            flags: reader.read_u16()?,
            protocol: reader.read_u8()?,
            algorithm: Algorithm::from(reader.read_u8()?)?,
            public_key: reader.read(reader.unread_bytes())?,
        })
    }
}
