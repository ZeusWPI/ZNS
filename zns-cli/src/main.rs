use base64::prelude::*;
use num_bigint::BigUint;
use num_traits::FromPrimitive;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::str::from_utf8;
use zns::{errors::ZNSError, reader::Reader};

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    key: String,

    /// Name of the person to greet
    #[arg(short, long)]
    username: String,
}

pub trait KeyTransformer {
    fn from_openssh(reader: &mut Reader) -> Result<Self, ZNSError>
    where
        Self: Sized;

    fn to_dnskey(&self, username: &str) -> (String, String);
}

struct Ed25519KeyPair {
    private_payload: [u8; 32],
    public_payload: [u8; 32],
}

struct RSAKeyPair {
    modulus: Vec<u8>,
    public_exponent: Vec<u8>,
    private_exponent: Vec<u8>,
    prime1: Vec<u8>,
    prime2: Vec<u8>,
    exponent1: Vec<u8>,
    exponent2: Vec<u8>,
    coefficient: Vec<u8>,
}

enum KeyPair {
    ED255519(Ed25519KeyPair),
    Rsa(RSAKeyPair),
}

#[allow(dead_code)]
struct OpenSSHKey {
    ciphername: String,
    kdfname: String,
    kdfoptions: String,
    keypair: KeyPair,
}

fn read_string(reader: &mut Reader) -> Result<String, ZNSError> {
    let length = reader.read_u32()?;
    let data = reader.read(length as usize)?;
    let result = from_utf8(&data).map_err(|e| ZNSError::Key {
        message: format!("Wrong ciphername format: {}", e),
    })?;
    Ok(result.to_owned())
}

fn read_bytes(reader: &mut Reader) -> Result<Vec<u8>, ZNSError> {
    let length = reader.read_u32()?;
    let data = reader.read(length as usize)?;
    Ok(data)
}

impl KeyTransformer for Ed25519KeyPair {
    fn from_openssh(reader: &mut Reader) -> Result<Self, ZNSError> {
        // public key parts//TODO: change to SIG
        let length = reader.read_u32()?;
        reader.read(length as usize)?;

        // private key payload
        let length = reader.read_u32()?;
        let data = reader.read(length as usize)?;

        let private_payload = data[0..32].try_into().unwrap();
        let public_payload = data[32..].try_into().unwrap();

        Ok(Self {
            public_payload,
            private_payload,
        })
    }

    fn to_dnskey(&self, username: &str) -> (String, String) {
        let version: &str = "Private-key-format: v1.3";
        let algorithm: &str = "Algorithm: 15 (ED25519)";
        let private_key = format!(
            "PrivateKey: {}",
            BASE64_STANDARD.encode(self.private_payload)
        );
        let private_encoded = format!("{version}\n{algorithm}\n{private_key}");

        let public_key = BASE64_STANDARD.encode(self.public_payload);
        let public_encoded = format!("{username}.users.zeus.gent. IN KEY 256 3 15 {public_key}");

        (private_encoded, public_encoded)
    }
}

impl KeyTransformer for RSAKeyPair {
    fn from_openssh(reader: &mut Reader) -> Result<Self, ZNSError> {
        let mut modulus = read_bytes(reader)?;

        if modulus[0] == 0 {
            // Remove first byte, it's a null byte for sign.
            modulus.remove(0);
        }

        let public_exponent = read_bytes(reader)?;
        let private_exponent = read_bytes(reader)?;
        let coefficient = read_bytes(reader)?;
        let prime1 = read_bytes(reader)?;
        let prime2 = read_bytes(reader)?;

        let d = BigUint::from_bytes_be(&private_exponent);
        let p = BigUint::from_bytes_be(&prime1);
        let q = BigUint::from_bytes_be(&prime2);

        let pm = &d % (&p - BigUint::from_u8(1).unwrap());
        let qm = &d % (&q - BigUint::from_u8(1).unwrap());

        let exponent1 = pm.to_bytes_be();
        let exponent2 = qm.to_bytes_be();

        Ok(Self {
            modulus,
            public_exponent,
            private_exponent,
            prime1,
            prime2,
            exponent1,
            exponent2,
            coefficient,
        })
    }

    fn to_dnskey(&self, username: &str) -> (String, String) {
        let modulus = BASE64_STANDARD.encode(&self.modulus);
        let pubexponent = BASE64_STANDARD.encode(&self.public_exponent);
        let privexponent = BASE64_STANDARD.encode(&self.private_exponent);
        let prime1 = BASE64_STANDARD.encode(&self.prime1);
        let prime2 = BASE64_STANDARD.encode(&self.prime2);
        let exp1 = BASE64_STANDARD.encode(&self.exponent1);
        let exp2 = BASE64_STANDARD.encode(&self.exponent2);
        let coeff = BASE64_STANDARD.encode(&self.coefficient);

        let private_encoded = format!(
            "Private-key-format: v1.3
Algorithm: 10 (RSASHA512)
Modulus: {modulus}
PublicExponent: {pubexponent}
PrivateExponent: {privexponent}
Prime1: {prime1}
Prime2: {prime2}
Exponent1: {exp1}
Exponent2: {exp2}
Coefficient: {coeff}
"
        );

        let mut public_key: Vec<u8> = vec![];

        public_key.push(self.public_exponent.len() as u8);
        public_key.extend(&self.public_exponent);
        public_key.extend(&self.modulus);

        let encoded_pub = BASE64_STANDARD.encode(&public_key);

        let public_encoded = format!("{username}.users.zeus.gent. IN KEY 256 3 10 {encoded_pub}");

        (private_encoded, public_encoded)
    }
}

impl KeyTransformer for OpenSSHKey {
    fn from_openssh(reader: &mut Reader) -> Result<Self, ZNSError> {
        // Reference Material: https://coolaj86.com/articles/the-openssh-private-key-format/

        let buf = reader.read(14)?;
        let magic = from_utf8(&buf).map_err(|e| ZNSError::Key {
            message: format!("Not valid ASCII: {}", e),
        })?;

        if magic != "openssh-key-v1" {
            return Err(ZNSError::Key {
                message: String::from("ssh key does not match ASCII magic openssh-key-v1"),
            });
        }

        reader.read_u8()?;

        let ciphername = read_string(reader)?;
        let kdfname = read_string(reader)?;
        let kdfoptions = read_string(reader)?;

        // Amount of keypairs
        let nkeys = reader.read_u32()?;

        if nkeys != 1 {
            return Err(ZNSError::Key {
                message: format!(
                    "Only private key file with one keypair is supported, not {} keys",
                    nkeys
                ),
            });
        }

        // public key
        let length = reader.read_u32()?;
        reader.read(length as usize)?;

        // size of remaining payload
        reader.read_u32()?;

        // salt and rounds
        reader.read(8)?;

        // public keytype
        let keytype = read_string(reader)?;

        let keypair = match keytype.as_str() {
            "ssh-ed25519" => Ok(KeyPair::ED255519(Ed25519KeyPair::from_openssh(reader)?)),
            "ssh-rsa" => Ok(KeyPair::Rsa(RSAKeyPair::from_openssh(reader)?)),
            other => Err(ZNSError::Key {
                message: format!("Invalid public keytype {}", other),
            }),
        }?;

        let length = reader.read_u32()?;
        reader.read(length as usize)?;

        Ok(Self {
            ciphername,
            kdfname,
            kdfoptions,
            keypair,
        })
    }

    fn to_dnskey(&self, username: &str) -> (String, String) {
        match &self.keypair {
            KeyPair::ED255519(keypair) => keypair.to_dnskey(username),
            KeyPair::Rsa(keypair) => keypair.to_dnskey(username),
        }
    }
}

const OPENSSH_START: &str = "-----BEGIN OPENSSH PRIVATE KEY-----";
const OPENSSH_END: &str = "-----END OPENSSH PRIVATE KEY-----";
const FILENAME: &str = "Kdns";

fn ssh_to_dnskey(file_content: &str, username: &str) -> Result<(), Box<dyn Error>> {
    if !file_content.starts_with(OPENSSH_START) || !file_content.ends_with(OPENSSH_END) {
        Err(ZNSError::Key {
            message: format!(
                "file should start with {} and end with {}",
                OPENSSH_START, OPENSSH_END
            ),
        })?
    }

    let key_encoded = &file_content[OPENSSH_START.len()..file_content.len() - OPENSSH_END.len()]
        .replace('\n', "");

    let bin = BASE64_STANDARD.decode(key_encoded)?;
    let mut reader = Reader::new(&bin);
    let key = OpenSSHKey::from_openssh(&mut reader)?;

    let mut file_private = File::create(format!("{}.private", FILENAME))?;
    let mut file_public = File::create(format!("{}.key", FILENAME))?;

    let (private, public) = key.to_dnskey(username);
    file_private.write_all(private.as_bytes())?;
    file_public.write_all(public.as_bytes())?;

    Ok(())
}

fn main() {
    let args = Args::parse();

    match fs::read_to_string(args.key) {
        Ok(contents) => match ssh_to_dnskey(contents.trim(), &args.username) {
            Ok(()) => println!(
                "Successfully written {}.private and {}.key",
                FILENAME, FILENAME
            ),
            Err(error) => eprintln!("{}", error),
        },
        Err(error) => eprintln!("{}", error),
    }
}
