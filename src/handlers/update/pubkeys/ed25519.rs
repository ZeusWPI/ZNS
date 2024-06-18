use ring::signature;

use crate::reader::Reader;

use super::{PublicKey, PublicKeyError, SSH_ED25519};

pub struct Ed25519PublicKey {
    data: Vec<u8>,
}

impl PublicKey for Ed25519PublicKey {
    fn from_openssh(key: &[u8]) -> Result<Self, PublicKeyError>
    where
        Self: Sized,
    {
        let mut reader = Reader::new(key);
        Ed25519PublicKey::verify_ssh_type(&mut reader, SSH_ED25519)?;
        reader.read_i32()?;
        Ok(Ed25519PublicKey {
            data: reader.read(reader.unread_bytes())?,
        })
    }

    fn from_dnskey(key: &[u8]) -> Result<Self, PublicKeyError>
    where
        Self: Sized,
    {
        Ok(Ed25519PublicKey { data: key.to_vec() })
    }

    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, PublicKeyError> {
        let pkey = ring::signature::UnparsedPublicKey::new(&signature::ED25519, &self.data);

        Ok(pkey.verify(data, signature).is_ok())
    }
}
