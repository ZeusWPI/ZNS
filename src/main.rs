use std::{error::Error, net::SocketAddr};

use crate::{api::api_listener_loop, resolver::resolver_listener_loop};

mod api;
mod db;
mod errors;
mod parser;
mod resolver;
mod structs;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let resolver_add = SocketAddr::from(([127, 0, 0, 1], 8080));
    let (_, _) = tokio::join!(
        resolver_listener_loop(resolver_add),
        api_listener_loop(api_addr)
    );
    Ok(())
}
