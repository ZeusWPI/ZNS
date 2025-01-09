use diesel::PgConnection;
use reqwest::header::ACCEPT;

use crate::{config::Config, db::models::get_from_database};

use zns::{
    errors::ZNSError,
    labelstring::LabelString,
    parser::FromBytes,
    reader::Reader,
    structs::{Class, RRClass, RRType, Type},
};

use super::{dnskey::DNSKeyRData, sig::Sig};

pub async fn authenticate(
    sig: &Sig,
    zone: &LabelString,
    connection: &mut PgConnection,
) -> Result<bool, ZNSError> {
    if zone.len() > Config::get().authoritative_zone.len() {
        let ssh_verified = match &Config::get().zauth_url {
            Some(url) => {
                let username = &zone.as_slice()
                    [zone.as_slice().len() - Config::get().authoritative_zone.as_slice().len() - 1];

                validate_ssh(&username.to_lowercase(), url, sig)
                    .await
                    .map_err(|e| ZNSError::Servfail {
                        message: e.to_string(),
                    })?
            }
            None => false,
        };

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

async fn validate_ssh(
    username: &String,
    zauth_url: &String,
    sig: &Sig,
) -> Result<bool, reqwest::Error> {
    let client = reqwest::Client::new();
    Ok(client
        .get(format!("{}/users/{}/keys", zauth_url, username))
        .header(ACCEPT, "application/json")
        .send()
        .await?
        .json::<Vec<String>>()
        .await?
        .iter()
        .any(|key| match sig.verify_ssh(key) {
            Ok(value) => value,
            Err(e) => {
                eprintln!("{}", e);
                false
            }
        }))
}

async fn validate_dnskey(
    zone: &LabelString,
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
        let data: Vec<u8> = rr.rdata.clone().into();
        let mut reader = Reader::new(&data);
        DNSKeyRData::from_bytes(&mut reader).is_ok_and(|dnskey| match sig.verify_dnskey(dnskey) {
            Ok(value) => value,
            Err(e) => {
                eprintln!("{}", e);
                false
            }
        })
    }))
}
