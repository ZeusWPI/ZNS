use std::time::{SystemTime, UNIX_EPOCH};

use base64::prelude::*;
use int_enum::IntEnum;

use zns::{
    errors::ZNSError, labelstring::LabelString, parser::FromBytes, reader::Reader, structs::RR,
};

use super::{
    dnskey::DNSKeyRData,
    pubkeys::{Ed25519PublicKey, PublicKey, RsaPublicKey, SSH_ED25519, SSH_RSA},
};

pub struct Sig {
    raw_data: Vec<u8>,
    key_rdata: SigRData,
}

#[allow(dead_code)]
#[derive(Debug)]
struct SigRData {
    type_covered: u16,
    algo: Algorithm,
    labels: u8,
    original_ttl: u32,
    signature_expiration: u32,
    signature_inception: u32,
    key_tag: u16,
    signer: LabelString,
    signature: Vec<u8>,
}

/// https://www.iana.org/assignments/dns-sec-alg-numbers/dns-sec-alg-numbers.xhtml
#[repr(u8)]
#[derive(IntEnum, Debug, PartialEq)]
pub enum Algorithm {
    ED25519 = 15,
    RSASHA512 = 10,
    RSASHA256 = 8,
}

impl Algorithm {
    pub fn from(value: u8) -> Result<Self, ZNSError> {
        Algorithm::try_from(value).map_err(|a| ZNSError::NotImp {
            object: String::from("Algorithm"),
            message: format!("Usupported algorithm: {}", a),
        })
    }
}

impl FromBytes for SigRData {
    fn from_bytes(reader: &mut Reader) -> Result<Self, ZNSError> {
        if reader.unread_bytes() < 18 {
            Err(ZNSError::Parse {
                object: String::from("KeyRData"),
                message: String::from("invalid rdata"),
            })
        } else {
            Ok(SigRData {
                type_covered: reader.read_u16()?,
                algo: Algorithm::from(reader.read_u8()?)?,
                labels: reader.read_u8()?,
                original_ttl: reader.read_u32()?,
                signature_expiration: reader.read_u32()?,
                signature_inception: reader.read_u32()?,
                key_tag: reader.read_u16()?,
                signer: LabelString::from_bytes(reader)?,
                signature: reader.read(reader.unread_bytes())?,
            })
        }
    }
}

impl Sig {
    pub fn new(rr: &RR, datagram: &[u8]) -> Result<Self, ZNSError> {
        let mut request = datagram[0..datagram.len() - 11 - rr.rdlength as usize].to_vec();
        request[11] -= 1; // Decrease arcount

        let data: Vec<u8> = rr.rdata.clone().into();
        let mut reader = Reader::new(&data);
        let key_rdata = SigRData::from_bytes(&mut reader)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ZNSError::Servfail {
                message: e.to_string(),
            })?
            .as_secs();

        if (key_rdata.signature_inception as u64) > now {
            return Err(ZNSError::Refused {
                message: String::from("invalid signature inception time"),
            });
        }

        if (key_rdata.signature_expiration as u64) < now {
            return Err(ZNSError::Refused {
                message: String::from("signature has expired"),
            });
        }

        let mut raw_data = data[0..data.len() - key_rdata.signature.len()].to_vec();
        raw_data.extend(request);

        Ok(Sig {
            raw_data,
            key_rdata,
        })
    }

    fn verify(&self, key: impl PublicKey) -> Result<bool, ZNSError> {
        key.verify(
            &self.raw_data,
            &self.key_rdata.signature,
            &self.key_rdata.algo,
        )
    }

    pub fn verify_ssh(&self, key: &str) -> Result<bool, ZNSError> {
        let key_split: Vec<&str> = key.split_ascii_whitespace().collect();
        let bin = BASE64_STANDARD.decode(key_split[1]).unwrap();

        match (key_split[0], &self.key_rdata.algo) {
            (SSH_ED25519, Algorithm::ED25519) => self.verify(Ed25519PublicKey::from_openssh(&bin)?),
            (SSH_RSA, Algorithm::RSASHA512 | Algorithm::RSASHA256) => {
                self.verify(RsaPublicKey::from_openssh(&bin)?)
            }
            _ => Ok(false),
        }
    }

    pub fn verify_dnskey(&self, key: DNSKeyRData) -> Result<bool, ZNSError> {
        if self.key_rdata.algo != key.algorithm {
            Ok(false)
        } else {
            match self.key_rdata.algo {
                Algorithm::RSASHA512 | Algorithm::RSASHA256 => {
                    self.verify(RsaPublicKey::from_dnskey(&key.public_key)?)
                }
                Algorithm::ED25519 => self.verify(Ed25519PublicKey::from_dnskey(&key.public_key)?),
            }
        }
    }
}
