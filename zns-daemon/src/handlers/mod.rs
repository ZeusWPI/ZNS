use diesel::PgConnection;

use zns::{
    errors::ZNSError,
    structs::{Message, Opcode},
};

use crate::config::Config;

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
        // Check for a question the server is not autoritative for
        if let Some(qname) = message.not_authoritative(&Config::get().authoritative_zone) {
            return Err(ZNSError::NotAuth { message: qname });
        }

        match message.get_opcode() {
            //TODO: implement this in Opcode
            Ok(opcode) => match opcode {
                Opcode::QUERY => QueryHandler::handle(message, raw, connection).await,
                Opcode::UPDATE => UpdateHandler::handle(message, raw, connection).await,
            },
            Err(e) => Err(ZNSError::Formerr {
                message: e.to_string(),
            }),
        }
    }
}
