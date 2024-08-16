use diesel::prelude::*;
use diesel::sql_types::Text;
use zns::{
    errors::ZNSError,
    structs::{Class, Type, RR},
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
struct Record {
    pub name: String,
    pub _type: i32,
    pub class: i32,
    pub ttl: i32,
    pub rdlength: i32,
    pub rdata: Vec<u8>,
}

sql_function! {
    fn lower(x: Text) -> Text;
}

impl Record {
    pub fn get(
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

    pub fn create(
        db: &mut PgConnection,
        new_record: Record,
    ) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(records::table)
            .values(&new_record)
            .execute(db)
    }

    pub fn delete(
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

pub fn insert_into_database(rr: &RR, connection: &mut PgConnection) -> Result<(), ZNSError> {
    let record = Record {
        name: rr.name.join("."),
        _type: rr._type.clone().into(),
        class: rr.class.clone().into(),
        ttl: rr.ttl,
        rdlength: rr.rdlength as i32,
        rdata: rr.rdata.clone(),
    };

    Record::create(connection, record).map_err(|e| ZNSError::Servfail {
        message: e.to_string(),
    })?;

    Ok(())
}

pub fn get_from_database(
    name: &Vec<String>,
    _type: Option<Type>,
    class: Class,
    connection: &mut PgConnection,
) -> Result<Vec<RR>, ZNSError> {
    let records = Record::get(
        connection,
        name.join("."),
        _type.map(|t| t.into()),
        class.into(),
    )
    .map_err(|e| ZNSError::Servfail {
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
pub fn delete_from_database(
    name: &Vec<String>,
    _type: Option<Type>,
    class: Class,
    rdata: Option<Vec<u8>>,
    connection: &mut PgConnection,
) {
    let _ = Record::delete(
        connection,
        name.join("."),
        _type.map(|f| f.into()),
        class.into(),
        rdata,
    );
}

#[cfg(test)]
mod tests {

    use zns::test_utils::get_rr;

    use super::*;

    use crate::db::lib::tests::get_test_connection;

    #[test]
    fn test() {
        let mut connection = get_test_connection();

        let rr = get_rr();

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
            Some(rr.rdata.clone()),
            &mut connection,
        );

        assert!(f(&mut connection).unwrap().is_empty());

        assert!(insert_into_database(&rr, &mut connection).is_ok());
        assert!(insert_into_database(&rr, &mut connection).is_err());
    }
}
