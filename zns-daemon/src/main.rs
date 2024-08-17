use std::{error::Error, net::SocketAddr};

mod config;
mod db;
mod handlers;
mod resolver;
mod utils;

use config::Config;

use crate::resolver::{tcp_listener_loop, udp_listener_loop};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    Config::initialize();
    let resolver_add = SocketAddr::from((Config::get().address, Config::get().port));
    let _ = tokio::join!(
        udp_listener_loop(resolver_add),
        tcp_listener_loop(resolver_add)
    );
    Ok(())
}
