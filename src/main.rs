use std::{error::Error, net::SocketAddr};

use crate::resolver::resolver_listener_loop;

mod db;
mod errors;
mod parser;
mod resolver;
mod auth;
mod structs;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let resolver_add = SocketAddr::from(([127, 0, 0, 1], 8080));
    let _ = tokio::join!(
        resolver_listener_loop(resolver_add),
    );
    Ok(())
}
