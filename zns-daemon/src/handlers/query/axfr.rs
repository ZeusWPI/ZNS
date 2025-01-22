use zns::{
    errors::ZNSError,
    structs::{RRType, Type, RR},
};

use crate::{auth::verify_authorization, db::models::Record, handlers::ResponseHandler};

use super::get_default_soa;

pub struct AXFRHandler {}

impl ResponseHandler for AXFRHandler {
    async fn handle(
        message: &zns::structs::Message,
        raw: &[u8],
        connection: &mut diesel::PgConnection,
    ) -> Result<zns::structs::Message, zns::errors::ZNSError> {
        let mut response = message.clone();

        if message.header.qdcount != 1 {
            return Err(ZNSError::Refused {
                message: "QDCOUNT must be one".to_string(),
            });
        }

        let question = &message.question[0];
        let zone = &question.qname;

        if !verify_authorization(message, zone, raw, connection).await? {
            return Err(ZNSError::Refused {
                message: "Not Authorized".to_string(),
            });
        }

        //TODO: TC header flag MUST be 0

        let results = Record::get_by_suffix(
            connection,
            &zone.to_string(),
            question.qclass.clone().into(),
        )
        .map_err(|e| ZNSError::Servfail {
            message: e.to_string(),
        })?;

        let rrs: Vec<RR> = results
            .into_iter()
            .filter_map(|record| record.into())
            .filter(|rr: &RR| rr._type != Type::Type(RRType::SOA))
            .collect();

        let soa = get_default_soa(zone)?;

        response.extend_answer(vec![soa.clone()]);
        response.extend_answer(rrs);
        response.extend_answer(vec![soa]);

        Ok(response)
    }
}
