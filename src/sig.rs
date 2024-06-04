use base64::prelude::*;

use crate::{
    parser::FromBytes,
    structs::{KeyRData, RR},
};

pub struct Sig {
    raw_data: Vec<u8>,
    key_rdata: KeyRData,
}

pub enum PublicKey {
    ED25519(String),
}

impl Sig {
    pub fn new(rr: &RR, datagram: &[u8]) -> Sig {
        let mut request = datagram[0..datagram.len() - 11 - rr.rdlength as usize].to_vec();
        request[11] -= 1; // Decrease arcount

        let mut i = 0;
        let key_rdata = KeyRData::from_bytes(&rr.rdata, &mut i).unwrap();

        let mut raw_data = rr.rdata[0..i].to_vec();
        raw_data.extend(request);

        Sig {
            raw_data,
            key_rdata,
        }
    }

    fn verify_ed25519(&self, key: String) -> bool {
        let blob = BASE64_STANDARD.decode(key).unwrap();

        let pkey = ring::signature::UnparsedPublicKey::new(
            &ring::signature::ED25519,
            &blob.as_slice()[19..],
        );

        pkey.verify(&self.raw_data, &self.key_rdata.signature)
            .is_ok()
    }

    pub fn verify(&self, key: PublicKey) -> bool {
        match key {
            PublicKey::ED25519(pkey) => self.verify_ed25519(pkey),
        }
    }
}
