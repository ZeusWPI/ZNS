use crate::{db::models::get_from_database, errors::ZNSError, structs::Message};

use super::ResponseHandler;

pub struct QueryHandler {}

impl ResponseHandler for QueryHandler {
    async fn handle(message: &Message, _raw: &[u8]) -> Result<Message, ZNSError> {
        let mut response = message.clone();
        response.header.arcount = 0; //TODO: fix this, handle unknown class values

        for question in &message.question {
            let answers = get_from_database(
                &question.qname,
                question.qtype.clone(),
                question.qclass.clone(),
            )
            .await;

            match answers {
                Ok(rrs) => {
                    if rrs.len() == 0 {
                        return Err(ZNSError::NXDomain {
                            domain: question.qname.join("."),
                        });
                    }
                    response.header.ancount = rrs.len() as u16;
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
