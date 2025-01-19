use diesel::PgConnection;

use zns::{
    errors::ZNSError,
    structs::{Message, Question, RRType, Type, RR},
};

use crate::{config::Config, db::models::get_from_database};

use super::{get_default_soa, ResponseHandler};

pub struct NormalQueryHandler {}

//TODO: the clones in this file should and could be avoided
impl ResponseHandler for NormalQueryHandler {
    async fn handle(
        message: &Message,
        _raw: &[u8],
        connection: &mut PgConnection,
    ) -> Result<Message, ZNSError> {
        let mut response = message.clone();

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

                        rrs.extend(try_cname(&domain_records));

                        if domain_records.is_empty() && !question.qname.is_empty() {
                            rrs.extend(try_wildcard(question, connection)?);
                        }

                        if rrs.is_empty()
                            && question.qtype == Type::Type(RRType::SOA)
                            && Config::get().default_soa
                        {
                            rrs.extend([get_default_soa(&question.qname)?])
                        }

                        if rrs.is_empty() && domain_records.is_empty() {
                            return Err(ZNSError::NXDomain {
                                domain: question.qname.to_string(),
                                qtype: question.qtype.clone(),
                            });
                        }
                    }

                    response.extend_answer(rrs);
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

fn try_cname(records: &[RR]) -> Vec<RR> {
    records
        .iter()
        .filter(|rr| rr._type == Type::Type(RRType::CNAME))
        .cloned()
        .collect()
}

fn try_wildcard(question: &Question, connection: &mut PgConnection) -> Result<Vec<RR>, ZNSError> {
    let mut qname = question.qname.clone().to_vec();
    qname[0] = String::from("*");
    let matches: Vec<RR> = get_from_database(
        &qname.clone().into(),
        Some(question.qtype.clone()),
        question.qclass.clone(),
        connection,
    )?
    .into_iter()
    .map(|mut rr| {
        rr.name.clone_from(&question.qname);
        rr
    })
    .collect();

    // Maybe wildcard cname exists
    if matches.is_empty() {
        Ok(get_from_database(
            &qname.into(),
            Some(Type::Type(RRType::CNAME)),
            question.qclass.clone(),
            connection,
        )?
        .into_iter()
        .map(|mut rr| {
            rr.name.clone_from(&question.qname);
            rr
        })
        .collect())
    } else {
        Ok(matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::db::{lib::tests::get_test_connection, models::insert_into_database};
    use zns::{
        parser::ToBytes,
        test_utils::{get_cname_rr, get_message, get_rr},
    };

    #[tokio::test]
    async fn test_handle_query() {
        let mut connection = get_test_connection();
        let rr = get_rr(Some(Config::get().authoritative_zone.clone()));
        let mut message = get_message(Some(Config::get().authoritative_zone.clone()));
        message.header.ancount = 0;
        message.answer = vec![];

        assert!(insert_into_database(&rr, &mut connection).is_ok());

        let result = NormalQueryHandler::handle(
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

        let result = NormalQueryHandler::handle(
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

    #[tokio::test]
    async fn test_cname() {
        let mut connection = get_test_connection();
        let rr = get_cname_rr(Some(Config::get().authoritative_zone.clone()));

        assert!(insert_into_database(&rr, &mut connection).is_ok());

        let mut message = get_message(Some(Config::get().authoritative_zone.clone()));
        message.header.ancount = 0;
        message.answer = vec![];

        let result = NormalQueryHandler::handle(
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
    async fn test_cname_wildcard_query() {
        let mut connection = get_test_connection();

        let wildcard = Config::get().authoritative_zone.prepend("*".to_string());
        let non_existent = Config::get()
            .authoritative_zone
            .prepend("nonexistent".to_string());

        let mut rr = get_cname_rr(Some(wildcard));

        let mut message = get_message(Some(non_existent.clone()));
        message.header.ancount = 0;
        message.answer = vec![];

        assert!(insert_into_database(&rr, &mut connection).is_ok());

        rr.name = non_existent;

        let result = NormalQueryHandler::handle(
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
