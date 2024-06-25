use crate::{
    errors::ZNSError,
    structs::{Message, Opcode},
};

use self::{query::QueryHandler, update::UpdateHandler};

mod query;
mod update;

pub trait ResponseHandler {
    async fn handle(message: &Message, raw: &[u8]) -> Result<Message, ZNSError>;
}

pub struct Handler {}

impl ResponseHandler for Handler {
    async fn handle(message: &Message, raw: &[u8]) -> Result<Message, ZNSError> {
        match message.get_opcode() {
            Ok(opcode) => match opcode {
                Opcode::QUERY => QueryHandler::handle(&message, raw).await,
                Opcode::UPDATE => UpdateHandler::handle(&message, raw).await,
            },
            Err(e) => Err(ZNSError::Formerr {
                message: e.to_string(),
            }),
        }
    }
}
