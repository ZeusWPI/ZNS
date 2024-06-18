use base64::prelude::*;

use crate::{
    config::Config,
    db::models::get_from_database,
    errors::{AuthenticationError, DatabaseError},
    parser::FromBytes,
    reader::Reader,
    structs::{Class, RRClass, RRType, Type},
};

use super::{
    dnskey::DNSKeyRData,
    pubkeys::{Ed25519PublicKey, PublicKey, PublicKeyError, RsaPublicKey, SSH_ED25519, SSH_RSA},
    sig::Sig,
};

pub(super) async fn authenticate(
    sig: &Sig,
    zone: &Vec<String>,
) -> Result<bool, AuthenticationError> {
    if zone.len() >= 4 {
        let username = &zone[zone.len() - 4]; // Should match: username.users.zeus.gent

        let ssh_verified = validate_ssh(username, sig).await?;

        if ssh_verified {
            Ok(true)
        } else {
            Ok(validate_dnskey(zone, sig).await?)
        }
    } else {
        Err(AuthenticationError {
            message: String::from("Invalid zone"),
        })
    }
}

async fn validate_ssh(username: &String, sig: &Sig) -> Result<bool, PublicKeyError> {
    Ok(reqwest::get(format!(
        "{}/users/keys/{}",
        Config::get().zauth_url,
        username
    ))
    .await?
    .json::<Vec<String>>()
    .await?
    .iter()
    .any(|key| {
        let key_split: Vec<&str> = key.split_ascii_whitespace().collect();
        let bin = BASE64_STANDARD.decode(key_split[1]).unwrap();
        match key_split[0] {
            //TODO: do something with error, debugging?
            SSH_ED25519 => {
                Ed25519PublicKey::from_openssh(&bin).is_ok_and(|pubkey| sig.verify(pubkey))
            }
            SSH_RSA => RsaPublicKey::from_openssh(&bin).is_ok_and(|pubkey| sig.verify(pubkey)),
            _ => false,
        }
    }))
}

async fn validate_dnskey(zone: &Vec<String>, sig: &Sig) -> Result<bool, DatabaseError> {
    Ok(
        get_from_database(zone, Type::Type(RRType::DNSKEY), Class::Class(RRClass::IN))
            .await?
            .iter()
            .any(|rr| {
                let mut reader = Reader::new(&rr.rdata);
                DNSKeyRData::from_bytes(&mut reader)
                    .map(|key| key.verify(sig))
                    .is_ok_and(|b| b)
            }),
    )
}

impl From<reqwest::Error> for PublicKeyError {
    fn from(value: reqwest::Error) -> Self {
        PublicKeyError {
            message: format!("Reqwest Error: {}", value.to_string()),
        }
    }
}

impl From<PublicKeyError> for AuthenticationError {
    fn from(value: PublicKeyError) -> Self {
        AuthenticationError {
            message: value.to_string(),
        }
    }
}
