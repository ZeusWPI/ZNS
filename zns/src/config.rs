use std::{env, sync::OnceLock};

use dotenvy::dotenv;

static CONFIG: OnceLock<Config> = OnceLock::new();

pub struct Config {
    pub zauth_url: String,
    pub db_uri: String,
    pub authoritative_zone: Vec<String>,
}

impl Config {
    pub fn initialize() {
        assert!(CONFIG.get().is_none());

        Config::get();
    }

    pub fn get() -> &'static Config {
        CONFIG.get_or_init(|| {
            dotenv().ok();
            Config {
                db_uri: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
                zauth_url: env::var("ZAUTH_URL").expect("ZAUTH_URL must be set"),
                authoritative_zone: env::var("ZONE")
                    .expect("ZONE must be set")
                    .split(".")
                    .map(str::to_string)
                    .collect(),
            }
        })
    }
}
