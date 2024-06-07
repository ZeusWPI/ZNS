use std::{error::Error, net::SocketAddr};

use dotenvy::dotenv;

use crate::resolver::resolver_listener_loop;

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
    dotenv().ok();

    let resolver_add = SocketAddr::from(([127, 0, 0, 1], 8080));
    let _ = tokio::join!(resolver_listener_loop(resolver_add),);
    Ok(())
}
