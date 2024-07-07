use diesel::PgConnection;

use crate::{
    errors::ZNSError,
    structs::{Message, Opcode},
};

use self::{query::QueryHandler, update::UpdateHandler};

mod query;
mod update;

pub trait ResponseHandler {
    async fn handle(
        message: &Message,
        raw: &[u8],
        connection: &mut PgConnection,
    ) -> Result<Message, ZNSError>;
}

pub struct Handler {}

impl ResponseHandler for Handler {
    async fn handle(
        message: &Message,
        raw: &[u8],
        connection: &mut PgConnection,
    ) -> Result<Message, ZNSError> {
        match message.get_opcode() {
            //TODO: implement this in Opcode
            Ok(opcode) => match opcode {
                Opcode::QUERY => QueryHandler::handle(&message, raw, connection).await,
                Opcode::UPDATE => UpdateHandler::handle(&message, raw, connection).await,
            },
            Err(e) => Err(ZNSError::Formerr {
                message: e.to_string(),
            }),
        }
    }
}
