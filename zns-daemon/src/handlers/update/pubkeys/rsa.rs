use crate::handlers::update::sig::Algorithm;
use ring::signature;
use zns::{errors::ZNSError, reader::Reader};

use super::{PublicKey, SSH_RSA};

pub struct RsaPublicKey {
    e: Vec<u8>,
    n: Vec<u8>,
}

#[derive(asn1::Asn1Write)]
struct RsaAsn1<'a> {
    n: Option<asn1::BigInt<'a>>,
    e: Option<asn1::BigInt<'a>>,
}

impl PublicKey for RsaPublicKey {
    fn from_openssh(key: &[u8]) -> Result<Self, ZNSError>
    where
        Self: Sized,
    {
        let mut reader = Reader::new(key);
        RsaPublicKey::verify_ssh_type(&mut reader, SSH_RSA)?;
        let e_size = reader.read_i32()?;
        let e = reader.read(e_size as usize)?;
        let n_size = reader.read_i32()?;
        let n = reader.read(n_size as usize)?;
        Ok(RsaPublicKey { e, n })
    }

    fn from_dnskey(key: &[u8]) -> Result<Self, ZNSError>
    where
        Self: Sized,
    {
        let mut reader = Reader::new(key);
        let e_len = reader.read_u8()?;
        let e = reader.read(e_len as usize)?;
        let mut n = reader.read(reader.unread_bytes())?;
        n.insert(0, 0);
        Ok(RsaPublicKey { e, n })
    }

    fn verify(
        &self,
        data: &[u8],
        signature: &[u8],
        algorithm: &Algorithm,
    ) -> Result<bool, ZNSError> {
        let result = asn1::write_single(&RsaAsn1 {
            n: asn1::BigInt::new(&self.n),
            e: asn1::BigInt::new(&self.e),
        })
        .map_err(|e| ZNSError::Key {
            message: format!("Verify Error: {}", e),
        })?;

        let signature_type = match algorithm {
            Algorithm::RSASHA512 => Ok(&signature::RSA_PKCS1_2048_8192_SHA512),
            Algorithm::RSASHA256 => Ok(&signature::RSA_PKCS1_2048_8192_SHA256),
            _ => Err(ZNSError::Key {
                message: String::from("RsaPublicKey: invalid verify algorithm"),
            }),
        }?;

        let pkey = ring::signature::UnparsedPublicKey::new(signature_type, result);

        Ok(pkey.verify(data, signature).is_ok())
    }
}
