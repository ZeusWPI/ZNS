// @generated automatically by Diesel CLI.

diesel::table! {
    records (name, type_, class, rdlength, rdata) {
        name -> Text,
        #[sql_name = "type"]
        type_ -> Int4,
        class -> Int4,
        ttl -> Int4,
        rdlength -> Int4,
        rdata -> Bytea,
    }
}
