use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::UdpSocket;

use crate::db::{lib::establish_connection, models::Record};
use crate::errors::DatabaseError;
use crate::parser::FromBytes;
use crate::structs::{Message, Question};

use crate::structs::{Class, Type, RR};
const MAX_DATAGRAM_SIZE: usize = 40_96;

async fn get_from_database(question: Question) -> Result<RR, DatabaseError> {
    let db_connection = &mut establish_connection();
    let record = Record::get(
        db_connection,
        question.qname.join("."),
        question.qtype as i32,
        question.qclass as i32,
    )
    .map_err(|e| DatabaseError {
        message: e.to_string(),
    })?;

    Ok(RR {
        name: record.name.split(".").map(str::to_string).collect(),
        _type: Type::try_from(record._type as u16).map_err(|e| DatabaseError { message: e })?,
        class: Class::try_from(record.class as u16).map_err(|e| DatabaseError { message: e })?,
        ttl: record.ttl,
        rdlength: record.rdlength as u16,
        rdata: record.rdata,
    })
}

async fn insert_into_database(rr: RR) -> Result<(), DatabaseError> {
    let db_connection = &mut establish_connection();
    let record = Record {
        name: rr.name.join("."),
        _type: rr._type as i32,
        class: rr.class as i32,
        ttl: rr.ttl,
        rdlength: rr.rdlength as i32,
        rdata: rr.rdata,
    };

    Record::create(db_connection, record).map_err(|e| DatabaseError {
        message: e.to_string(),
    })?;

    Ok(())
}

async fn create_query(message: Message) -> Message {
    let mut response = message.clone();

    let answer = get_from_database(message.question).await;

    match answer {
        Ok(rr) => {
            response.header.flags |= 0b1000010110000000;
            response.header.ancount = 1;
            response.header.arcount = 0;
            response.answer = Some(rr)
        }
        Err(e) => {
            response.header.flags |= 0b1000010110000011;
            eprintln!("{}", e);
        }
    }

    response
}

pub async fn resolver_listener_loop(addr: SocketAddr) -> Result<(), Box<dyn Error>> {
    let socket_shared = Arc::new(UdpSocket::bind(addr).await?);
    loop {
        let mut data = vec![0u8; MAX_DATAGRAM_SIZE];
        let (len, addr) = socket_shared.recv_from(&mut data).await?;
        match Message::from_bytes(&data[..len]) {
            Ok(message) => {
                let socket = socket_shared.clone();
                tokio::spawn(async move {
                    let response = create_query(message).await;
                    let _ = socket
                        .send_to(Message::to_bytes(response).as_slice(), addr)
                        .await;
                });
            }
            Err(err) => println!("{}", err),
        };
    }
}
