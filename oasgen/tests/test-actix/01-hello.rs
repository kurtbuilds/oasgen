use oasgen::{OaSchema, Server, oasgen};
use actix_web::web::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, OaSchema)]
pub struct SendCode {
    pub mobile: String,
}

#[derive(Serialize, OaSchema)]
pub struct SendCodeResponse {
    pub found_account: bool,
}

#[oasgen]
async fn send_code(_body: Json<SendCode>) -> Json<SendCodeResponse> {
    Json(SendCodeResponse { found_account: false })
}

fn main() {
    use pretty_assertions::assert_eq;
    let server = Server::actix()
        .post("/hello", send_code)
        ;
    let spec = serde_yaml::to_string(&server.openapi).unwrap();
    let other = include_str!("01-hello.yaml");
    assert_eq!(spec.trim(), other);
}