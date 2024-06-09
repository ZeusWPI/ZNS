use base64::prelude::*;

use crate::{
    errors::ParseError,
    parser::FromBytes,
    reader::Reader,
    structs::{KeyRData, RR},
};

pub(super) struct Sig {
    raw_data: Vec<u8>,
    key_rdata: KeyRData,
}

impl Sig {
    pub fn new(rr: &RR, datagram: &[u8]) -> Result<Sig, ParseError> {
        let mut request = datagram[0..datagram.len() - 11 - rr.rdlength as usize].to_vec();
        request[11] -= 1; // Decrease arcount

        let mut reader = Reader::new(&rr.rdata);
        let key_rdata = KeyRData::from_bytes(&mut reader)?;

        let mut raw_data = rr.rdata[0..rr.rdata.len() - key_rdata.signature.len()].to_vec();
        raw_data.extend(request);

        Ok(Sig {
            raw_data,
            key_rdata,
        })
    }

    pub fn verify_ed25519(&self, key: &str) -> bool {
        let blob = BASE64_STANDARD.decode(key).unwrap();

        let pkey = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, &blob);

        pkey.verify(&self.raw_data, &self.key_rdata.signature)
            .is_ok()
    }

    pub fn verify_ssh_ed25519(&self, key: &str) -> bool {
        let blob = BASE64_STANDARD.decode(key).unwrap();

        let pkey = ring::signature::UnparsedPublicKey::new(
            &ring::signature::ED25519,
            &blob.as_slice()[19..],
        );

        pkey.verify(&self.raw_data, &self.key_rdata.signature)
            .is_ok()
    }
}
