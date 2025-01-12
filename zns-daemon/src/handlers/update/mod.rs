use diesel::PgConnection;

use crate::{
    config::Config,
    db::models::{delete_from_database, get_from_database, insert_into_database},
};

use zns::structs::{Class, Message, RRClass, RRType, Type};
use zns::{errors::ZNSError, structs::RR};

use self::sig::Sig;

use super::ResponseHandler;

mod authenticate;
mod dnskey;
mod pubkeys;
mod sig;

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

        // Check Zone authority
        message.check_authoritative(&Config::get().authoritative_zone)?;

        // Check Prerequisite    TODO: implement this

        let zone = &message.question[0];
        let zlen = zone.qname.as_slice().len();

        //TODO: this code is ugly
        let last = message.additional.last();
        if last.is_some() && last.unwrap()._type == Type::Type(RRType::SIG) {
            let sig = Sig::new(last.unwrap(), raw)?;

            if !authenticate::authenticate(&sig, &zone.qname, connection).await? {
                return Err(ZNSError::Refused {
                    message: "Unable to verify authentication".to_string(),
                });
            }
        } else {
            return Err(ZNSError::Refused {
                message: "No KEY record at the end of request found".to_string(),
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
    let rr_type = match record._type {
        Type::Type(RRType::CNAME) => None,
        _ => Some(Type::Type(RRType::CNAME)),
    };

    let records = get_from_database(&record.name, rr_type, record.class.clone(), connection)?;
    if !records.is_empty() {
        Ok(Some(
            "Another record with the same name already exists".to_string(),
        ))
    } else {
        Ok(None)
    }
}
