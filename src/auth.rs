use std::{
    fs::{read_to_string, File},
    io::{Read, Write},
};

use base64::prelude::*;
use ring::signature::Ed25519KeyPair;

pub fn verify(user: String, signature: &[u8], message: &[u8]) -> bool {
    let str = read_to_string("/home/xander/Desktop/dnsclient/dns.pub").unwrap();
    let key_split: Vec<&str> = str.split_ascii_whitespace().collect();
    let blob = BASE64_STANDARD.decode(key_split[1]).unwrap();

    let mut prev = vec![ 0x30, 0x2a, 0x30,0x05, 0x06,0x03,0x2b,0x65, 0x70, 0x03, 0x21, 0x00];
    prev.extend_from_slice(&blob.as_slice()[19..]);
    let s = prev.as_slice();
    println!("{:#?}", &blob.as_slice()[19..]);


    let mut file = File::create("foo.txt").unwrap();
    file.write_all(s);

    let mut pem = File::open("/home/xander/Desktop/dnsclient/cert.der").unwrap();
    let mut pem_buf = Vec::<u8>::new();
    pem.read_to_end(&mut pem_buf).unwrap();
    let key = Ed25519KeyPair::from_pkcs8_maybe_unchecked(&pem_buf).unwrap();
    let mut pem = File::open("/home/xander/Desktop/dnsclient/der").unwrap();
    let mut pem_buf = Vec::<u8>::new();
    pem.read_to_end(&mut pem_buf).unwrap();

    // let rng = rand::SystemRandom::new();
    // let mut signature = vec![];
    // key.sign(&signature::RSA_PKCS1_SHA256, &rng, MESSAGE, &mut signature);
    let k  = ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, &blob.as_slice()[19..]);
    println!("{:#?}",k.verify(message, signature.as_ref()));
    

    return false;
}
