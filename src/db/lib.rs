use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::config::Config;

pub fn establish_connection() -> SqliteConnection {
    let database_url = Config::get().db_uri.clone();
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", Config::get().db_uri))
}
