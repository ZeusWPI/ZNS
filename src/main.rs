use std::{error::Error, net::SocketAddr};

use config::Config;

use crate::resolver::{tcp_listener_loop, udp_listener_loop};

mod config;
mod db;
mod errors;
mod handlers;
mod message;
mod parser;
mod reader;
mod resolver;
mod structs;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    Config::initialize();
    let resolver_add = SocketAddr::from(([127, 0, 0, 1], 5353));
    let _ = tokio::join!(
        udp_listener_loop(resolver_add),
        tcp_listener_loop(resolver_add)
    );
    Ok(())
}
