#![allow(unused)]

use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use actix_web::{FromRequest, Handler, Responder, Route, web};
use oasgen::{OaSchema, Server, openapi};
use actix_web::web::Json;
use http::Method;
use serde::{Deserialize, Serialize};

#[derive(OaSchema, Deserialize)]
pub struct SendCode {
    pub mobile: String,
}

#[derive(OaSchema, Deserialize)]
pub struct VerifyCode {
    pub mobile: String,
    pub code: String,
}

#[derive(Serialize, OaSchema, Debug)]
pub struct SendCodeResponse {
    pub found_account: bool,
}

#[openapi]
async fn send_code(_body: Json<SendCode>) -> Json<SendCodeResponse> {
    Json(SendCodeResponse { found_account: false })
}

#[openapi]
async fn verify_code(_body: Json<VerifyCode>) -> Json<()> {
    Json(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    use std::fs::File;
    use actix_web::{HttpResponse, web, HttpServer, App};
    // serde_yaml::to_writer(&File::create("examples/simple.yaml").unwrap(), &s.openapi).unwrap();

    let host = "0.0.0.0";
    let port = 5000;
    let host = format!("{}:{}", host, port);

    let server = Server::new()
        .post("/send-code", send_code)
        .post("/verify-code", verify_code)
        ;

    HttpServer::new(move || App::new()
        .route("/healthcheck", web::get().to(|| async { HttpResponse::Ok().body("Ok") }))
        .service(server.clone().into_service())
    )
        .bind(host)?
        .run()
        .await
}