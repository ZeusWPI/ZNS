use crate::{
    errors::DNSError,
    structs::{Message, Opcode, RCODE},
};

use self::{query::QueryHandler, update::UpdateHandler};

mod query;
mod update;

pub trait ResponseHandler {
    async fn handle(message: &Message, raw: &[u8]) -> Result<Message, DNSError>;
}

pub struct Handler {}

impl ResponseHandler for Handler {
    async fn handle(message: &Message, raw: &[u8]) -> Result<Message, DNSError> {
        match message.get_opcode() {
            Ok(opcode) => match opcode {
                Opcode::QUERY => QueryHandler::handle(&message, raw).await,
                Opcode::UPDATE => UpdateHandler::handle(&message, raw).await,
            },
            Err(e) => Err(DNSError {
                message: e.to_string(),
                rcode: RCODE::FORMERR,
            }),
        }
    }
}
