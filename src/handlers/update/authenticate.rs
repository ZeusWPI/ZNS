use std::env;

use reqwest::Error;

use crate::errors::AuthenticationError;

use super::sig::{PublicKey, Sig};

type SSHKeys = Vec<String>;

type Result<T> = std::result::Result<T, AuthenticationError>;

pub(super) async fn authenticate(sig: &Sig, zone: &Vec<String>) -> Result<bool> {
    if zone.len() >= 4 {
        let username = &zone[zone.len() - 4]; // Should match: username.users.zeus.gent
        let public_keys = get_keys(username).await.map_err(|e| AuthenticationError {
            message: e.to_string(),
        })?;

        Ok(public_keys.iter().any(|key| {
            let key_split: Vec<&str> = key.split_ascii_whitespace().collect();
            match key_split.len() {
                3 => {
                    let key_encoded = key_split[1].to_string();
                    match key_split[0] {
                        "ssh-ed25519" => sig.verify(PublicKey::ED25519(key_encoded)),
                        _ => false,
                    }
                }
                _ => false,
            }
        }))
    } else {
        Err(AuthenticationError {
            message: String::from("Invalid zone"),
        })
    }
}

async fn get_keys(username: &String) -> std::result::Result<SSHKeys, Error> {
    let zauth_url = env::var("ZAUTH_URL").expect("ZAUTH_URL must be set");
    Ok(
        reqwest::get(format!("{}/users/keys/{}", zauth_url, username))
            .await?
            .json::<SSHKeys>()
            .await?,
    )
}
