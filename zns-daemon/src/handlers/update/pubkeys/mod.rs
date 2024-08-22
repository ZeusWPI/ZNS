mod ed25519;
mod rsa;
use std::str::from_utf8;

use zns::{errors::ZNSError, reader::Reader};

pub use self::ed25519::Ed25519PublicKey;
pub use self::rsa::RsaPublicKey;

use super::sig::Algorithm;

pub const SSH_ED25519: &str = "ssh-ed25519";
pub const SSH_RSA: &str = "ssh-rsa";

pub trait PublicKey {
    fn verify_ssh_type(reader: &mut Reader, key_type: &str) -> Result<(), ZNSError> {
        let type_size = reader.read_i32()?;
        let read = reader.read(type_size as usize)?;
        let algo_type = from_utf8(&read).map_err(|e| ZNSError::Key {
            message: format!(
                "Could not convert type name bytes to string: {}",
                e
            ),
        })?;

        if algo_type == key_type {
            Ok(())
        } else {
            Err(ZNSError::Key {
                message: String::from("ssh key type does not match identifier"),
            })
        }
    }

    fn from_openssh(key: &[u8]) -> Result<Self, ZNSError>
    where
        Self: Sized;

    fn from_dnskey(key: &[u8]) -> Result<Self, ZNSError>
    where
        Self: Sized;

    fn verify(
        &self,
        data: &[u8],
        signature: &[u8],
        algorithm: &Algorithm,
    ) -> Result<bool, ZNSError>;
}
