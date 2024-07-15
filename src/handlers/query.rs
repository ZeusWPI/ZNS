use diesel::PgConnection;

use crate::{
    db::models::get_from_database,
    errors::ZNSError,
    structs::{Message, Question, RR},
};

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

        for question in &message.question {
            let answers = get_from_database(
                &question.qname,
                Some(question.qtype.clone()),
                question.qclass.clone(),
                connection,
            );

            match answers {
                Ok(mut rrs) => {
                    if rrs.len() == 0 {
                        rrs.extend(try_wildcard(question, connection)?);
                        if rrs.len() == 0 {
                            return Err(ZNSError::NXDomain {
                                domain: question.qname.join("."),
                            });
                        }
                    }
                    response.header.ancount += rrs.len() as u16;
                    response.answer.extend(rrs)
                }
                Err(e) => {
                    return Err(ZNSError::Database {
                        message: e.to_string(),
                    })
                }
            }
        }

        Ok(response)
    }
}

fn try_wildcard(question: &Question, connection: &mut PgConnection) -> Result<Vec<RR>, ZNSError> {
    let records = get_from_database(&question.qname, None, question.qclass.clone(), connection)?;

    if records.len() > 0 {
        Ok(vec![])
    } else {
        let mut qname = question.qname.clone();
        qname[0] = String::from("*");
        get_from_database(
            &qname,
            Some(question.qtype.clone()),
            question.qclass.clone(),
            connection,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tests::get_message;
    use crate::structs::*;

    use crate::{
        db::{lib::tests::get_test_connection, models::insert_into_database},
        parser::{tests::get_rr, ToBytes},
    };

    #[tokio::test]
    async fn test_handle_query() {
        let mut connection = get_test_connection();
        let rr = get_rr();
        let mut message = get_message();
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
}
