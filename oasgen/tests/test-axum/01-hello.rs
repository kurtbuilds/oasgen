use oasgen::{OaSchema, Server, openapi};
use axum::Json;
use serde::{Deserialize, Serialize};

/// Send a code to a mobile number
#[derive(Deserialize, OaSchema)]
pub struct SendCode {
    pub mobile: String,
}

#[derive(Serialize, OaSchema)]
pub struct SendCodeResponse {
    pub found_account: bool,
}

/// Endpoint to login by sending a code to the given mobile number
#[openapi(tags("auth"), summary = "A shorter description")]
async fn send_code(_body: Json<SendCode>) -> Json<SendCodeResponse> {
    Json(SendCodeResponse { found_account: false })
}

fn main() {
    use pretty_assertions::assert_eq;
    let server = Server::axum()
        .post("/hello", send_code)
        .freeze()
        ;

    let spec = server.openapi.as_ref().clone();
    let other = include_str!("01-hello.yaml");
    let other = serde_yaml::from_str::<oasgen::OpenAPI>(other).unwrap();
    assert_eq!(spec, other);
    let router = axum::Router::new()
        .merge(server.into_router());
    router.into_make_service();
}