#![allow(unused)]
use oasgen::{OaSchema, Server, openapi};
use actix_web::web::Json;
use serde::{Deserialize, Serialize};

#[derive(OaSchema, Deserialize)]
pub struct SendCode {
    pub mobile: String,
}

#[derive(Serialize, OaSchema, Debug)]
pub struct SendCodeResponse {
    pub found_account: bool,
}

#[openapi]
async fn send_code(_body: Json<SendCode>) -> Json<SendCodeResponse> {
    Json(SendCodeResponse { found_account: false })
}

use actix_web::dev::ServiceResponse;
use actix_web::HttpResponse;
use actix_web::dev::ServiceRequest;
use actix_web::Responder;
use actix_web::FromRequest;
use actix_web::Handler;

fn collector<F, Args>(handler: F) -> () where
    F: Handler<Args>,
    Args: FromRequest + 'static,
    F::Output: Responder + 'static,
{
    move |req: ServiceRequest| {
        let handler = handler.clone();
        async move {
            let (req, mut payload) = req.into_parts();

            let res = match Args::from_request(&req, &mut payload).await {
                Err(err) => HttpResponse::from_error(err),

                Ok(data) => handler
                    .call(data)
                    .await
                    .respond_to(&req)
                    .map_into_boxed_body(),
            };

            Ok(ServiceResponse::new(req, res))
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    use std::fs::File;
    use actix_web::{HttpResponse, web, HttpServer, App};
    // serde_yaml::to_writer(&File::create("examples/simple.yaml").unwrap(), &s.openapi).unwrap();

    let host = "0.0.0.0";
    let port = 5000;
    let host = format!("{}:{}", host, port);

    collector(send_code);

    // let server = Server::new()
    //     .post("/send-code", send_code)
    //     ;
        // .into_service();

    // HttpServer::new(move || App::new()
    // App::new()
    //     .route("/healthcheck", web::get().to(|| async { HttpResponse::Ok().body("Ok") }))
    //     .route("/send-code", web::post().to(send_code))
    // ;
    Ok(())
        // .service(server.clone().create_service())
        // .service(build_openapi().into_service("/api"))
        // .add_routes()
        // .wrap_api()
        // .route("/auth/send-code", post().to(auth::send_code))
                    // .with_json_spec_at("openapi.json")
                    // .build()
    // )
    //     .bind(host)?
    //     .run()
    //     .await
}