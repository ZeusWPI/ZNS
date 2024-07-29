use std::{error::Error, net::SocketAddr};

use config::Config;
use zns::config;

use zns::resolver::{tcp_listener_loop, udp_listener_loop};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    Config::initialize();
    let resolver_add = SocketAddr::from(([127, 0, 0, 1], 8080));
    let _ = tokio::join!(
        udp_listener_loop(resolver_add),
        tcp_listener_loop(resolver_add)
    );
    Ok(())
}
