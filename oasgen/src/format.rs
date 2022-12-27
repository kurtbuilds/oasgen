use std::convert::Infallible;
use std::future::{Ready, ready};
use actix_web::{FromRequest, HttpRequest};
use actix_web::body::BoxBody;
use actix_web::dev::Payload;

#[derive(Debug)]
pub enum Format {
    Json,
    Yaml,
    Html,
    Plain,
}

impl Format {
    pub fn sync_from_req(req: &HttpRequest) -> Self {
        if req.path().ends_with(".json") {
            return Format::Json;
        } else if req.path().ends_with(".yaml") {
            return Format::Json;
        }
        if let Some(accept) = req.headers().get("Accept") {
            let accept = accept.to_str().unwrap();
            if accept.contains("text/html") {
                return Format::Html;
            } else if accept.contains("application/json") {
                return Format::Json;
            }
        }
        Format::Plain
    }
}

impl FromRequest for Format {
    type Error = Infallible;
    type Future = Ready<Result<Self, Infallible>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        ready(Ok(Self::sync_from_req(req)))
    }
}