use crate::{
    db::models::get_from_database,
    errors::DNSError,
    structs::{Message, RCODE},
};

use super::ResponseHandler;

pub(super) struct QueryHandler {}

impl ResponseHandler for QueryHandler {
    async fn handle(message: &Message, _raw: &[u8]) -> Result<Message, DNSError> {
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
                    response.header.ancount = rrs.len() as u16;
                    response.answer.extend(rrs)
                }
                Err(e) => {
                    return Err(DNSError {
                        rcode: RCODE::NXDOMAIN,
                        message: e.to_string(),
                    })
                }
            }
        }

        Ok(response)
    }
}
