use diesel::prelude::*;

use crate::config::Config;

pub fn establish_connection() -> PgConnection {
    let database_url = Config::get().db_uri.clone();
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", Config::get().db_uri))
}
