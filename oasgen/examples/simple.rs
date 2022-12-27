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

#[tokio::main]
async fn main() -> std::io::Result<()> {
    use std::fs::File;
    use actix_web::{HttpResponse, web, HttpServer, App};

    let s = Server::new()
        .get("/hello", send_code)

        ;
    let res = send_code(Json(SendCode { mobile: "123".to_string() })).await;
    println!("{:#?}", res);
    serde_yaml::to_writer(&File::create("examples/simple.yaml").unwrap(), &s.openapi).unwrap();

    let host = "0.0.0.0";
    let port = 5000;
    let host = format!("{}:{}", host, port);

    let server = Server::new()
        .post("/send-code", send_code)
        ;
        // .into_service();

    HttpServer::new(move || App::new()
        .route("/healthcheck", web::get().to(|| async { HttpResponse::Ok().body("Ok") }))
        .route("/send-code", web::post().to(send_code))
        // .service(server.clone().create_service())
        // .service(build_openapi().into_service("/api"))
        // .add_routes()
        // .wrap_api()
        // .route("/auth/send-code", post().to(auth::send_code))
                    // .with_json_spec_at("openapi.json")
                    // .build()
    )
        .bind(host)?
        .run()
        .await
}