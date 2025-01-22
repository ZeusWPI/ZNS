use diesel::PgConnection;

use crate::auth::verify_authorization;
use crate::db::models::{delete_from_database, get_from_database, insert_into_database};

use zns::structs::{Class, Message, RRClass, RRType, Type};
use zns::{errors::ZNSError, structs::RR};

use super::ResponseHandler;

// Types which are not allowed to add. Array should be small.
static ILLEGAL_TYPES: [RRType; 2] = [RRType::SOA, RRType::NS];

pub struct UpdateHandler {}

impl ResponseHandler for UpdateHandler {
    async fn handle(
        message: &Message,
        raw: &[u8],
        connection: &mut PgConnection,
    ) -> Result<Message, ZNSError> {
        let response = message.clone();
        // Zone section (question) processing
        if (message.header.qdcount != 1)
            || !matches!(message.question[0].qtype, Type::Type(RRType::SOA))
        {
            return Err(ZNSError::Formerr {
                message: "Qdcount not one".to_string(),
            });
        }

        // Check Prerequisite    TODO: implement this

        let zone = &message.question[0];
        let zlen = zone.qname.as_slice().len();

        if !verify_authorization(message, &zone.qname, raw, connection).await? {
            return Err(ZNSError::Refused {
                message: "Not Authorized".to_string(),
            });
        }

        // Update Section Prescan
        for rr in &message.authority {
            let rlen = rr.name.as_slice().len();

            // Check if rr has same zone
            if rlen < zlen || !(zone.qname == rr.name.as_slice()[rlen - zlen..].into()) {
                return Err(ZNSError::Refused {
                    message: "RR has different zone from Question".to_string(),
                });
            }

            match (rr.class == Class::Class(RRClass::ANY) && (rr.ttl != 0 || rr.rdlength != 0))
                || (rr.class == Class::Class(RRClass::NONE) && rr.ttl != 0)
                || ![
                    Class::Class(RRClass::NONE),
                    Class::Class(RRClass::ANY),
                    zone.qclass.clone(),
                ]
                .contains(&rr.class)
            {
                true => {
                    return Err(ZNSError::Formerr {
                        message: "RR has invalid rr,ttl or class".to_string(),
                    });
                }
                false => (),
            }
        }

        for rr in &message.authority {
            if rr.class == zone.qclass {
                if let Some(message) = validate_record(rr, connection)? {
                    return Err(ZNSError::Refused { message });
                }
                insert_into_database(rr, connection)?;
            } else if rr.class == Class::Class(RRClass::ANY) {
                if rr._type == Type::Type(RRType::ANY) {
                    if rr.name == zone.qname {
                        return Err(ZNSError::NotImp {
                            object: String::from("Update Handler"),
                            message: "rr.name == zone.qname".to_string(),
                        });
                    } else {
                        delete_from_database(
                            &rr.name,
                            None,
                            Class::Class(RRClass::IN),
                            None,
                            connection,
                        )
                    }
                } else {
                    delete_from_database(
                        &rr.name,
                        Some(rr._type.clone()),
                        Class::Class(RRClass::IN),
                        None,
                        connection,
                    )
                }
            } else if rr.class == Class::Class(RRClass::NONE) {
                if rr._type == Type::Type(RRType::SOA) {
                    continue;
                }
                delete_from_database(
                    &rr.name,
                    Some(rr._type.clone()),
                    Class::Class(RRClass::IN),
                    Some(rr.rdata.clone().into()),
                    connection,
                )
            }
        }

        Ok(response)
    }
}

fn validate_record(record: &RR, connection: &mut PgConnection) -> Result<Option<String>, ZNSError> {
    if let Type::Type(rr_type) = &record._type {
        if ILLEGAL_TYPES.contains(rr_type) {
            return Ok(Some(format!("Illegal type in add: {:#?} ", rr_type)));
        }
    }

    let lookup_type = match record._type {
        Type::Type(RRType::CNAME) => None,
        _ => Some(Type::Type(RRType::CNAME)),
    };

    let records = get_from_database(&record.name, lookup_type, record.class.clone(), connection)?;
    if !records.is_empty() {
        Ok(Some(
            "Another record with the same name already exists".to_string(),
        ))
    } else {
        Ok(None)
    }
}
