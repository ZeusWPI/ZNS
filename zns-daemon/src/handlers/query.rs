use diesel::PgConnection;

use zns::{
    errors::ZNSError,
    labelstring::LabelString,
    parser::ToBytes,
    structs::{Class, Message, Question, RData, RRClass, RRType, SoaRData, Type, RR},
};

use crate::{config::Config, db::models::get_from_database};

use super::ResponseHandler;

pub struct QueryHandler {}

//TODO: the clones in this file should and could be avoided
impl ResponseHandler for QueryHandler {
    async fn handle(
        message: &Message,
        _raw: &[u8],
        connection: &mut PgConnection,
    ) -> Result<Message, ZNSError> {
        let mut response = message.clone();

        message.check_authoritative(&Config::get().authoritative_zone)?;

        for question in &message.question {
            let answers = get_from_database(
                &question.qname,
                Some(question.qtype.clone()),
                question.qclass.clone(),
                connection,
            );

            match answers {
                Ok(mut rrs) => {
                    if rrs.is_empty() {
                        let domain_records = get_from_database(
                            &question.qname,
                            None,
                            question.qclass.clone(),
                            connection,
                        )?;

                        if domain_records.is_empty() && !question.qname.is_empty() {
                            rrs.extend(try_wildcard(question, connection)?);
                        }

                        if rrs.is_empty()
                            && question.qtype == Type::Type(RRType::SOA)
                            && Config::get().default_soa
                        {
                            rrs.extend([get_soa(&question.qname)?])
                        }

                        if rrs.is_empty() && domain_records.is_empty() {
                            return Err(ZNSError::NXDomain {
                                domain: question.qname.to_string(),
                                qtype: question.qtype.clone(),
                            });
                        }
                    }
                    response.header.ancount += rrs.len() as u16;
                    response.answer.extend(rrs)
                }
                Err(e) => {
                    return Err(ZNSError::Servfail {
                        message: e.to_string(),
                    })
                }
            }
        }

        Ok(response)
    }
}

fn try_wildcard(question: &Question, connection: &mut PgConnection) -> Result<Vec<RR>, ZNSError> {
    let mut qname = question.qname.clone().to_vec();
    qname[0] = String::from("*");
    Ok(get_from_database(
        &qname.into(),
        Some(question.qtype.clone()),
        question.qclass.clone(),
        connection,
    )?
    .into_iter()
    .map(|mut rr| {
        rr.name.clone_from(&question.qname);
        println!("{:#?}", rr);
        rr
    })
    .collect())
}

fn get_soa(name: &LabelString) -> Result<RR, ZNSError> {
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
    } else if name.len() > auth_zone.len() {
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::db::{lib::tests::get_test_connection, models::insert_into_database};
    use zns::{
        parser::ToBytes,
        test_utils::{get_message, get_rr},
    };

    #[tokio::test]
    async fn test_handle_query() {
        let mut connection = get_test_connection();
        let rr = get_rr(Some(Config::get().authoritative_zone.clone()));
        let mut message = get_message(Some(Config::get().authoritative_zone.clone()));
        message.header.ancount = 0;
        message.answer = vec![];

        assert!(insert_into_database(&rr, &mut connection).is_ok());

        let result = QueryHandler::handle(
            &message,
            &Message::to_bytes(message.clone()),
            &mut connection,
        )
        .await
        .unwrap();
        assert_eq!(result.header.ancount, 2);
        assert_eq!(result.answer.len(), 2);
        assert_eq!(result.answer[0], rr);
        assert_eq!(result.answer[1], rr);
    }

    #[tokio::test]
    async fn test_wildcard_query() {
        let mut connection = get_test_connection();

        let wildcard = Config::get().authoritative_zone.prepend("*".to_string());
        let non_existent = Config::get()
            .authoritative_zone
            .prepend("nonexistent".to_string());

        let mut rr = get_rr(Some(wildcard));

        let mut message = get_message(Some(non_existent.clone()));
        message.header.ancount = 0;
        message.answer = vec![];

        assert!(insert_into_database(&rr, &mut connection).is_ok());

        rr.name = non_existent;

        let result = QueryHandler::handle(
            &message,
            &Message::to_bytes(message.clone()),
            &mut connection,
        )
        .await
        .unwrap();
        assert_eq!(result.header.ancount, 2);
        assert_eq!(result.answer.len(), 2);
        assert_eq!(result.answer[0], rr);
    }
}
