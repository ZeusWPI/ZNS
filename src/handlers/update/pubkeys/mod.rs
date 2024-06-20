mod ed25519;
mod rsa;
use core::fmt;
use std::str::from_utf8;

use crate::{errors::ReaderError, reader::Reader};

pub use self::ed25519::Ed25519PublicKey;
pub use self::rsa::RsaPublicKey;

use super::sig::Algorithm;

#[derive(Debug)]
pub struct PublicKeyError {
    pub message: String,
}

impl fmt::Display for PublicKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Public Key Error: {}", self.message)
    }
}

impl From<ReaderError> for PublicKeyError {
    fn from(value: ReaderError) -> Self {
        PublicKeyError {
            message: value.to_string(),
        }
    }
}

pub const SSH_ED25519: &str = "ssh-ed25519";
pub const SSH_RSA: &str = "ssh-rsa";

pub trait PublicKey {
    fn verify_ssh_type(reader: &mut Reader, key_type: &str) -> Result<(), PublicKeyError> {
        let type_size = reader.read_i32()?;
        let read = reader.read(type_size as usize)?;
        let algo_type = from_utf8(&read).map_err(|e| PublicKeyError {
            message: format!(
                "Could not convert type name bytes to string: {}",
                e.to_string()
            ),
        })?;

        if algo_type == key_type {
            Ok(())
        } else {
            Err(PublicKeyError {
                message: String::from("ssh key type does not match identifier"),
            })
        }
    }

    fn from_openssh(key: &[u8]) -> Result<Self, PublicKeyError>
    where
        Self: Sized;

    fn from_dnskey(key: &[u8]) -> Result<Self, PublicKeyError>
    where
        Self: Sized;

    fn verify(
        &self,
        data: &[u8],
        signature: &[u8],
        algorithm: &Algorithm,
    ) -> Result<bool, PublicKeyError>;
}
