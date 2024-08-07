use diesel::PgConnection;
use reqwest::header::ACCEPT;

use crate::{
    config::Config,
    db::models::get_from_database,
    errors::ZNSError,
    parser::FromBytes,
    reader::Reader,
    structs::{Class, RRClass, RRType, Type},
};

use super::{dnskey::DNSKeyRData, sig::Sig};

pub async fn authenticate(
    sig: &Sig,
    zone: &Vec<String>,
    connection: &mut PgConnection,
) -> Result<bool, ZNSError> {
    if zone.len() >= Config::get().authoritative_zone.len() {
        let username = &zone[zone.len() - Config::get().authoritative_zone.len() - 1];

        let ssh_verified = validate_ssh(username, sig).await.is_ok_and(|b| b);

        if ssh_verified {
            Ok(true)
        } else {
            Ok(validate_dnskey(zone, sig, connection).await?)
        }
    } else {
        Err(ZNSError::NotAuth {
            message: String::from("Invalid zone"),
        })
    }
}

async fn validate_ssh(username: &String, sig: &Sig) -> Result<bool, reqwest::Error> {
    let client = reqwest::Client::new();
    Ok(client
        .get(format!(
            "{}/users/{}/keys",
            Config::get().zauth_url,
            username
        ))
        .header(ACCEPT, "application/json")
        .send()
        .await?
        .json::<Vec<String>>()
        .await?
        .iter()
        .any(|key| sig.verify_ssh(&key).is_ok_and(|b| b)))
}

async fn validate_dnskey(
    zone: &Vec<String>,
    sig: &Sig,
    connection: &mut PgConnection,
) -> Result<bool, ZNSError> {
    Ok(get_from_database(
        zone,
        Some(Type::Type(RRType::DNSKEY)),
        Class::Class(RRClass::IN),
        connection,
    )?
    .iter()
    .any(|rr| {
        let mut reader = Reader::new(&rr.rdata);
        DNSKeyRData::from_bytes(&mut reader)
            .is_ok_and(|dnskey| sig.verify_dnskey(dnskey).is_ok_and(|b| b))
    }))
}
