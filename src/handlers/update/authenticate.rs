use reqwest::Error;

use crate::{
    config::Config,
    db::models::get_from_database,
    errors::{AuthenticationError, DatabaseError},
    parser::FromBytes,
    reader::Reader,
    structs::{Class, RRClass, RRType, Type},
};

use super::{dnskey::DNSKeyRData, sig::Sig};

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

async fn validate_ssh(username: &String, sig: &Sig) -> Result<bool, Error> {
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
        match key_split.len() {
            3 => match key_split[0] {
                "ssh-ed25519" => sig.verify_ssh_ed25519(key_split[1]),
                _ => false,
            },
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
