use diesel::prelude::*;
use diesel::sql_types::Text;
use zns::{
    errors::ZNSError,
    labelstring::LabelString,
    structs::{Class, RData, Type, RR},
};

use self::schema::records::{self};

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
pub struct Record {
    pub name: String,
    pub _type: i32,
    pub class: i32,
    pub ttl: i32,
    pub rdlength: i32,
    pub rdata: Vec<u8>,
}

define_sql_function! {
    fn lower(x: Text) -> Text;
}

impl Record {
    fn get(
        db: &mut PgConnection,
        name: String,
        _type: Option<i32>,
        class: i32,
    ) -> Result<Vec<Record>, diesel::result::Error> {
        let mut query = records::table.into_boxed();

        query = query.filter(
            lower(records::name)
                .eq(name.to_lowercase())
                .and(records::class.eq(class)),
        );

        if let Some(value) = _type {
            query = query.filter(records::_type.eq(value))
        }

        query.get_results(db)
    }

    // Returns all records with names ending with given suffix.
    pub fn get_by_suffix(
        db: &mut PgConnection,
        suffix: &str,
        class: i32,
    ) -> Result<Vec<Record>, diesel::result::Error> {
        let query = records::table
            .filter(
                records::name
                    .ilike(format!("%{}", suffix))
                    .and(records::class.eq(class)),
            )
            .order(records::name.desc());
        query.get_results(db)
    }

    fn create(db: &mut PgConnection, new_record: Record) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(records::table)
            .values(&new_record)
            .execute(db)
    }

    fn delete(
        db: &mut PgConnection,
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

const MAX_RDATA_SIZE: usize = 1000;

pub fn insert_into_database(rr: &RR, connection: &mut PgConnection) -> Result<(), ZNSError> {
    if rr.rdata.len() > MAX_RDATA_SIZE {
        return Err(ZNSError::Refused {
            message: format!(
                "RDATA size of record is bigger then maximum limit: {}",
                MAX_RDATA_SIZE
            ),
        });
    }

    let record = Record {
        name: rr.name.to_string(),
        _type: rr._type.clone().into(),
        class: rr.class.clone().into(),
        ttl: rr.ttl,
        rdlength: rr.rdlength as i32,
        rdata: rr.rdata.clone().into(),
    };

    Record::create(connection, record).map_err(|e| ZNSError::Servfail {
        message: e.to_string(),
    })?;

    Ok(())
}

pub fn get_from_database(
    name: &LabelString,
    _type: Option<Type>,
    class: Class,
    connection: &mut PgConnection,
) -> Result<Vec<RR>, ZNSError> {
    let records = Record::get(
        connection,
        name.to_string(),
        _type.map(|t| t.into()),
        class.into(),
    )
    .map_err(|e| ZNSError::Servfail {
        message: e.to_string(),
    })?;

    Ok(records
        .into_iter()
        .filter_map(|record| record.into())
        .collect())
}

//TODO: cleanup models
pub fn delete_from_database(
    name: &LabelString,
    _type: Option<Type>,
    class: Class,
    rdata: Option<Vec<u8>>,
    connection: &mut PgConnection,
) {
    let _ = Record::delete(
        connection,
        name.to_string(),
        _type.map(|f| f.into()),
        class.into(),
        rdata,
    );
}

impl From<Record> for Option<RR> {
    fn from(record: Record) -> Self {
        RData::from_safe(&record.rdata, &Type::from(record._type as u16))
            .map(|rdata| RR {
                name: LabelString::from(&record.name),
                _type: Type::from(record._type as u16),
                class: Class::from(record.class as u16),
                ttl: record.ttl,
                rdlength: record.rdlength as u16,
                rdata,
            })
            .ok()
    }
}

#[cfg(test)]
mod tests {

    use zns::test_utils::get_rr;

    use super::*;

    use crate::db::lib::tests::get_test_connection;

    #[test]
    fn test() {
        let mut connection = get_test_connection();

        let rr = get_rr(None);

        let f = |connection: &mut PgConnection| {
            get_from_database(
                &rr.name,
                Some(rr._type.clone()),
                rr.class.clone(),
                connection,
            )
        };

        assert!(f(&mut connection).unwrap().is_empty());

        assert!(insert_into_database(&rr, &mut connection).is_ok());

        let result = f(&mut connection);
        assert!(result.is_ok());
        assert_eq!(result.as_ref().unwrap().len(), 1);
        assert_eq!(result.unwrap()[0], rr);

        delete_from_database(
            &rr.name,
            Some(rr._type.clone()),
            rr.class.clone(),
            Some(rr.rdata.clone().into()),
            &mut connection,
        );

        assert!(f(&mut connection).unwrap().is_empty());

        assert!(insert_into_database(&rr, &mut connection).is_ok());
        assert!(insert_into_database(&rr, &mut connection).is_err());
    }
}
