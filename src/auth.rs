use std::fs::read_to_string;

use base64::prelude::*;

pub fn verify(signature: &[u8], message: &[u8]) -> bool {

    let str = read_to_string("dns.pub").unwrap(); //TODO: pub ssh key use zauth

    let key_split: Vec<&str> = str.split_ascii_whitespace().collect();
    let blob = BASE64_STANDARD.decode(key_split[1]).unwrap();

    let key  = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, &blob.as_slice()[19..]);

    return key.verify(&message, signature.as_ref()).is_ok();
}
