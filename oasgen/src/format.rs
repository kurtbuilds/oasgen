use std::convert::Infallible;
use std::future::{Ready, ready};

#[derive(Debug)]
pub enum Format {
    Json,
    Yaml,
    Html,
    Plain,
}

impl Format {
    #[cfg(feature = "actix")]
    pub fn sync_from_req(req: &actix_web::HttpRequest) -> Self {
        if req.path().ends_with(".json") {
            return Format::Json;
        } else if req.path().ends_with(".yaml") {
            return Format::Yaml;
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

#[cfg(feature = "actix")]
impl actix_web::FromRequest for Format {
    type Error = Infallible;
    type Future = Ready<Result<Self, Infallible>>;

    fn from_request(req: &actix_web::HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        ready(Ok(Self::sync_from_req(req)))
    }
}