use std::collections::HashMap;
use std::net::SocketAddr;

use http_body_util::{BodyExt, Full};
use hyper::body::{Buf, Bytes};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use serde::Deserialize;
use tokio::net::TcpListener;

use crate::db::models::insert_into_database;
use crate::structs::{Class, Type, RR};

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, GenericError>;
type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

static INTERNAL_SERVER_ERROR: &[u8] = b"Internal Server Error";
static NOTFOUND: &[u8] = b"Not Found";

#[derive(Deserialize)]
struct Record {
    name: Vec<String>,
    #[serde(rename = "type")]
    _type: Type,
    ttl: i32,
    data: String,
}

async fn create_record(req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody>> {
    let whole_body = req.collect().await?.aggregate();

    match serde_json::from_reader::<_, Record>(whole_body.reader()) {
        Ok(record) => {
            let rdata = record
                ._type
                .to_data(&record.data)
                .map_err(|e| e.to_string())?;
            match insert_into_database(RR {
                name: record.name,
                _type: record._type,
                class: Class::IN,
                ttl: record.ttl,
                rdlength: rdata.len() as u16,
                rdata,
            })
            .await
            {
                Ok(_) => Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(full("Successfully Created"))?),
                Err(e) => {
                    eprintln!("{}", e.to_string());
                    Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(full(INTERNAL_SERVER_ERROR))?)
                }
            }
        }
        Err(e) => Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(full(e.to_string()))?),
    }
}

async fn get_record(req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody>> {
    if let Some(q) = req.uri().query() {
        let params = form_urlencoded::parse(q.as_bytes())
            .into_owned()
            .collect::<HashMap<String, String>>();
        if let Some(domain) = params.get("domain_name") {
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .body(full(domain.to_owned()))?);
        }
    }

    Ok(Response::builder()
        .status(StatusCode::UNPROCESSABLE_ENTITY)
        .body(full("Missing domain_name query parameter"))?)
}

async fn routes(req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody>> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/add") => create_record(req).await,
        (&Method::GET, "/get") => get_record(req).await,
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(full(NOTFOUND))
            .unwrap()),
    }
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

pub async fn api_listener_loop(
    addr: SocketAddr,
) -> Result<Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(routes))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
