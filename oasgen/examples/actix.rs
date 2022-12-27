#![allow(unused)]

// We have to wrap the example in `mod` beacuse examples fail compilation without a `main`, and
// forwarding to an inner mod fixes the issue.
#[cfg(feature = "actix")]
mod inner {
    use oasgen::{OaSchema, Server, openapi};
    use actix_web::web::Json;
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
    pub async fn main() -> std::io::Result<()> {
        use std::fs::File;
        use actix_web::{HttpResponse, web, HttpServer, App};

        let host = "0.0.0.0";
        let port = 5000;
        let host = format!("{}:{}", host, port);

        let server = Server::new()
            .post("/send-code", send_code)
            .post("/verify-code", verify_code)
            .route_yaml_spec("/openapi.yaml")
            .write_and_exit_if_env_var("./openapi.yaml")
            .freeze()
            ;

        println!("Listening on {}", host);
        HttpServer::new(move || {
            let spec = server.openapi.clone();
            App::new()
                // App::new()
                .route("/healthcheck", web::get().to(|| async { HttpResponse::Ok().body("Ok") }))
                .service(server.clone().into_service())
        })
            .bind(host)?
            .run()
            .await
    }
}

fn main() {
    #[cfg(feature = "actix")]
    inner::main().unwrap()
}