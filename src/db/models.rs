use self::schema::records;
use diesel::prelude::*;

mod schema {
    diesel::table! {
        records (name, _type, class) {
            name -> Text,
            #[sql_name = "type"]
            _type -> Integer,
            class -> Integer,
            ttl -> Integer,
            rdlength -> Integer,
            rdata -> Binary,
        }
    }
}

#[derive(Insertable, Queryable, Selectable)]
#[diesel(table_name = records)]
pub struct Record {
    pub name: String,
    pub _type: i32,
    pub class: i32,
    pub ttl: i32,
    pub rdlength: i32,
    pub rdata: Vec<u8>,
}

impl Record {
    pub fn get(
        db: &mut SqliteConnection,
        name: String,
        _type: i32,
        class: i32,
    ) -> Result<Record, diesel::result::Error> {
        records::table.find((name, _type, class)).get_result(db)
    }

    pub fn create(
        db: &mut SqliteConnection,
        new_record: Record,
    ) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(records::table)
            .values(&new_record)
            .execute(db)
    }
}
