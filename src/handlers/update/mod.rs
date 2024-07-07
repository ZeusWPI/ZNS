use diesel::PgConnection;

use crate::{
    db::models::{delete_from_database, insert_into_database},
    errors::ZNSError,
    structs::{Class, Message, RRClass, RRType, Type},
    utils::vec_equal,
};

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
        let zone = &message.question[0];
        let zlen = zone.qname.len();
        if !(zlen >= 2 && zone.qname[zlen - 1] == "gent" && zone.qname[zlen - 2] == "zeus") {
            return Err(ZNSError::Formerr {
                message: "Invalid zone".to_string(),
            });
        }

        // Check Prerequisite    TODO: implement this

        //TODO: this code is ugly
        let last = message.additional.last();
        if last.is_some() && last.unwrap()._type == Type::Type(RRType::KEY) {
            let sig = Sig::new(last.unwrap(), raw)?;

            if !authenticate::authenticate(&sig, &zone.qname, connection)
                .await
                .is_ok_and(|x| x)
            {
                return Err(ZNSError::NotAuth {
                    message: "Unable to verify authentication".to_string(),
                });
            }
        } else {
            return Err(ZNSError::NotAuth {
                message: "No KEY record at the end of request found".to_string(),
            });
        }

        // Update Section Prescan
        for rr in &message.authority {
            let rlen = rr.name.len();

            // Check if rr has same zone
            if rlen < zlen || !(vec_equal(&zone.qname, &rr.name[rlen - zlen..])) {
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
                let _ = insert_into_database(&rr, connection);
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
                    Some(rr.rdata.clone()),
                    connection,
                )
            }
        }

        Ok(response)
    }
}
