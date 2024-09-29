use std::{env, net::IpAddr, sync::OnceLock};

use dotenvy::dotenv;
use zns::labelstring::LabelString;

static CONFIG: OnceLock<Config> = OnceLock::new();

pub struct Config {
    pub zauth_url: Option<String>,
    pub db_uri: String,
    pub authoritative_zone: LabelString,
    pub port: u16,
    pub address: IpAddr,
    pub default_soa: bool,
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
                zauth_url: env::var("ZAUTH_URL").ok(),
                authoritative_zone: LabelString::from(&env::var("ZONE").expect("ZONE must be set")),
                port: env::var("ZNS_PORT")
                    .map(|v| v.parse::<u16>().expect("ZNS_PORT is invalid"))
                    .unwrap_or(5333),
                address: env::var("ZNS_ADDRESS")
                    .unwrap_or(String::from("127.0.0.1"))
                    .parse()
                    .expect("ZNS_ADDRESS is invalid"),
                default_soa: env::var("ZNS_DEFAULT_SOA")
                    .unwrap_or(String::from("true"))
                    .parse()
                    .expect("ZNS_DEFAULT_SOA should have value `true` or `false`"),
            }
        })
    }
}
