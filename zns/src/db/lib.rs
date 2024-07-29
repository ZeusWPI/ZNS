use diesel::prelude::*;

use crate::config::Config;

pub fn get_connection() -> PgConnection {
    let database_url = Config::get().db_uri.clone();
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", Config::get().db_uri))
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn get_test_connection() -> PgConnection {
        let mut connection = get_connection();
        assert!(connection.begin_test_transaction().is_ok());
        connection
    }
}
