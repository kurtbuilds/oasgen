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

    let host = "0.0.0.0";
    let port = 5000;
    let host = format!("{}:{}", host, port);

    let server = Server::new()
        .post("/send-code", send_code)
        .post("/verify-code", verify_code)
        ;
        // .into_service();

    HttpServer::new(move || {
        let spec = server.openapi.clone();
        let spec_json = serde_json::to_string(&spec).unwrap();
        let spec_yaml = serde_yaml::to_string(&spec).unwrap();
        App::new()
            // App::new()
            .route("/healthcheck", web::get().to(|| async { HttpResponse::Ok().body("Ok") }))
            .route("/openapi.json", web::get().to(move || {
                let spec_json = spec_json.clone();
                async { HttpResponse::Ok().insert_header(("Content-Type", "application/json")).body(spec_json) }
            }))
            .route("/openapi.yaml", web::get().to(move || {
                let spec = spec_yaml.clone();
                async { HttpResponse::Ok().body(spec) }
            }))
            .service(server.clone().into_service())
    })
    //     .route("/send-code", web::post().to(send_code))
    // ;
    // Ok(())
        // .service(server.clone().create_service())
        // .service(build_openapi().into_service("/api"))
        // .add_routes()
        // .wrap_api()
        // .route("/auth/send-code", post().to(auth::send_code))
                    // .with_json_spec_at("openapi.json")
                    // .build()
        .bind(host)?
        .run()
        .await
}
