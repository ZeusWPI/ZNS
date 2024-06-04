use crate::{
    errors::DatabaseError,
    structs::{Class, Question, Type, RR},
};
use diesel::prelude::*;

use self::schema::records::{self};

use super::lib::establish_connection;

mod schema {
    diesel::table! {
        records (name, _type, class, rdlength, rdata) {
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
    ) -> Result<Vec<Record>, diesel::result::Error> {
        records::table
            .filter(
                records::name
                    .eq(name)
                    .and(records::_type.eq(_type).and(records::class.eq(class))),
            )
            .get_results(db)
    }

    pub fn create(
        db: &mut SqliteConnection,
        new_record: Record,
    ) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(records::table)
            .values(&new_record)
            .execute(db)
    }

    pub fn delete(
        db: &mut SqliteConnection,
        name: String,
        _type: Option<i32>,
        class: i32,
        rdata: Option<Vec<u8>>,
    ) -> Result<usize, diesel::result::Error> {
        let mut query = diesel::delete(records::table).into_boxed();

        query = query.filter(records::name.eq(name).and(records::class.eq(class)));

        if let Some(_type) = _type {
            query = query.filter(records::_type.eq(_type));
        }

        if let Some(rdata) = rdata {
            query = query.filter(records::rdata.eq(rdata));
        }

        query.execute(db)
    }
}

pub async fn insert_into_database(rr: RR) -> Result<(), DatabaseError> {
    let db_connection = &mut establish_connection();
    let record = Record {
        name: rr.name.join("."),
        _type: rr._type.into(),
        class: rr.class.into(),
        ttl: rr.ttl,
        rdlength: rr.rdlength as i32,
        rdata: rr.rdata,
    };

    Record::create(db_connection, record).map_err(|e| DatabaseError {
        message: e.to_string(),
    })?;

    Ok(())
}

pub async fn get_from_database(question: &Question) -> Result<Vec<RR>, DatabaseError> {
    let db_connection = &mut establish_connection();
    let records = Record::get(
        db_connection,
        question.qname.join("."),
        question.qtype.clone().into(),
        question.qclass.clone().into(),
    )
    .map_err(|e| DatabaseError {
        message: e.to_string(),
    })?;

    Ok(records
        .into_iter()
        .filter_map(|record| {
            Some(RR {
                name: record.name.split(".").map(str::to_string).collect(),
                _type: Type::from(record._type as u16),
                class: Class::from(record.class as u16),
                ttl: record.ttl,
                rdlength: record.rdlength as u16,
                rdata: record.rdata,
            })
        })
        .collect())
}

//TODO: cleanup models
pub async fn delete_from_database(
    name: Vec<String>,
    _type: Option<Type>,
    class: Class,
    rdata: Option<Vec<u8>>,
) {
    let db_connection = &mut establish_connection();
    let _ = Record::delete(db_connection, name.join("."), _type.map(|f| f.into()), class.into(), rdata);
}
