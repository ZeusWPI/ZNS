use axfr::AXFRHandler;
use diesel::PgConnection;

use normal_query::NormalQueryHandler;
use zns::{
    errors::ZNSError,
    labelstring::LabelString,
    parser::ToBytes,
    structs::{Class, Message, RData, RRClass, RRType, SoaRData, Type, RR},
};

use crate::config::Config;

use super::ResponseHandler;

mod axfr;
mod normal_query;

pub struct QueryHandler {}

impl ResponseHandler for QueryHandler {
    async fn handle(
        message: &Message,
        raw: &[u8],
        connection: &mut PgConnection,
    ) -> Result<Message, ZNSError> {
        match message
            .question
            .first()
            .filter(|q| q.qtype == Type::Type(RRType::AXFR))
        {
            Some(_) => AXFRHandler::handle(message, raw, connection).await,
            None => NormalQueryHandler::handle(message, raw, connection).await,
        }
    }
}

fn get_default_soa(name: &LabelString) -> Result<RR, ZNSError> {
    let auth_zone = Config::get().authoritative_zone.clone();
    let rdata = if &auth_zone == name {
        // Recommended values taken from wikipedia: https://en.wikipedia.org/wiki/SOA_record
        Ok(SoaRData {
            mname: auth_zone,
            rname: LabelString::from("admin.zeus.ugent.be"),
            serial: 1,
            refresh: 86400,
            retry: 7200,
            expire: 3600000,
            minimum: 172800,
        })
    } else if name.len() == 1 + auth_zone.len() {
        let zone: LabelString = name.as_slice()[name.len() - auth_zone.len() - 1..].into();
        Ok(SoaRData {
            mname: auth_zone,
            rname: LabelString::from(&format!("{}.zeus.ugent.be", zone.as_slice()[0])),
            serial: 1,
            refresh: 86400,
            retry: 7200,
            expire: 3600000,
            minimum: 172800,
        })
    } else {
        Err(ZNSError::NXDomain {
            domain: name.to_string(),
            qtype: Type::Type(RRType::SOA),
        })
    }?;

    Ok(RR {
        name: name.to_owned(),
        _type: Type::Type(RRType::SOA),
        class: Class::Class(RRClass::IN),
        ttl: 11200,
        rdlength: 0,
        rdata: RData::from_safe(&SoaRData::to_bytes(rdata), &Type::Type(RRType::SOA))?,
    })
}
