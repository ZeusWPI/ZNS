// @generated automatically by Diesel CLI.

diesel::table! {
    records (name, type_, class, rdlength, rdata) {
        name -> Text,
        #[sql_name = "type"]
        type_ -> Integer,
        class -> Integer,
        ttl -> Integer,
        rdlength -> Integer,
        rdata -> Binary,
    }
}
