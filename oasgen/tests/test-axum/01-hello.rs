use oasgen::{OaSchema, Server, openapi};
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, OaSchema)]
pub struct SendCode {
    pub mobile: String,
}

#[derive(Serialize, OaSchema)]
pub struct SendCodeResponse {
    pub found_account: bool,
}

#[openapi]
async fn send_code(_body: Json<SendCode>) -> Json<SendCodeResponse> {
    Json(SendCodeResponse { found_account: false })
}

fn main() {
    use pretty_assertions::assert_eq;
    let server = Server::axum()
        .post("/hello", send_code)
        .freeze()
        ;

    let spec = server.openapi.clone();
    let spec = serde_yaml::to_string(&spec).unwrap();
    assert_eq!(spec.trim(), include_str!("01-hello.yaml"));
    let router = axum::Router::new()
        .merge(server.into_router());
    router.into_make_service();
}