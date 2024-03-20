use crate::{
    errors::DatabaseError,
    structs::{Class, Question, Type, RR},
};
use diesel::prelude::*;

use self::schema::records;

use super::lib::establish_connection;

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
struct Record {
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

pub async fn insert_into_database(rr: RR) -> Result<(), DatabaseError> {
    let db_connection = &mut establish_connection();
    let record = Record {
        name: rr.name.join("."),
        _type: rr._type as i32,
        class: rr.class as i32,
        ttl: rr.ttl,
        rdlength: rr.rdlength as i32,
        rdata: rr.rdata,
    };

    Record::create(db_connection, record).map_err(|e| DatabaseError {
        message: e.to_string(),
    })?;

    Ok(())
}

pub async fn get_from_database(question: &Question) -> Result<RR, DatabaseError> {
    let db_connection = &mut establish_connection();
    let record = Record::get(
        db_connection,
        question.qname.join("."),
        question.qtype.clone() as i32,
        question.qclass.clone() as i32,
    )
    .map_err(|e| DatabaseError {
        message: e.to_string(),
    })?;

    Ok(RR {
        name: record.name.split(".").map(str::to_string).collect(),
        _type: Type::try_from(record._type as u16).map_err(|e| DatabaseError { message: e })?,
        class: Class::try_from(record.class as u16).map_err(|e| DatabaseError { message: e })?,
        ttl: record.ttl,
        rdlength: record.rdlength as u16,
        rdata: record.rdata,
    })
}
