// We have to wrap the example in `mod` beacuse examples fail compilation without a `main`, and
// forwarding to an inner mod fixes the issue.
#[cfg(feature = "actix")]
#[cfg(feature = "swagger-ui")]
pub mod inner {
    use actix_web::{guard, web::Json};
    use oasgen::{
        actix::{post, scope},
        oasgen, OaSchema, Server,
    };
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

    #[oasgen]
    async fn send_code(_body: Json<SendCode>) -> Json<SendCodeResponse> {
        Json(SendCodeResponse {
            found_account: false,
        })
    }

    #[oasgen]
    async fn verify_code(_body: Json<VerifyCode>) -> Json<()> {
        Json(())
    }

    #[oasgen]
    async fn pong() -> Json<String> {
        Json("pong".into())
    }

    #[tokio::main]
    pub async fn main() -> std::io::Result<()> {
        use actix_web::{web, App, HttpResponse, HttpServer};

        let host = ("0.0.0.0", 5000);

        println!("Listening on {:?}", host);
        println!("Swagger http://{}:{}/docs/", host.0, host.1);

        HttpServer::new(move || {
            let server = Server::actix()
                .service(
                    scope("/code")
                        .route("/send-code", post().to(send_code))
                        .route("/verify-code", post().guard(guard::Post()).to(verify_code)),
                )
                .get("/ping", pong)
                .route_json_spec("/docs/openapi.json")
                .route_yaml_spec("/docs/openapi.yaml")
                .swagger_ui("/docs/")
                .write_and_exit_if_env_var_set("./openapi.yaml")
                .freeze();

            App::new()
                .route(
                    "/healthcheck",
                    web::get().to(|| async { HttpResponse::Ok().body("Ok") }),
                )
                .service(server.into_service())
        })
        .bind(host)?
        .run()
        .await
    }
}

fn main() {
    #[cfg(feature = "actix")]
    #[cfg(feature = "swagger-ui")]
    inner::main().unwrap()
}
