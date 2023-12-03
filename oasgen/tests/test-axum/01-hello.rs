use oasgen::{OaSchema, Server, oasgen};
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
#[oasgen(tags("auth"), summary = "A shorter description")]
async fn send_code(_body: Json<SendCode>) -> Json<SendCodeResponse> {
    Json(SendCodeResponse { found_account: false })
}

fn main() {
    use pretty_assertions::assert_eq;
    let server = Server::axum()
        .post("/hello", send_code)
        ;

    let spec = &server.openapi;
    let other = serde_yaml::from_str::<oasgen::OpenAPI>(include_str!("01-hello.yaml")).unwrap();
    assert_eq!(spec, &other);
    let router = axum::Router::new()
        .merge(server.freeze().into_router());
    router.into_make_service();
}