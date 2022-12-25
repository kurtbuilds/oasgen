use oasgen::{OaSchema, Server};
use actix_web::web::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, OaSchema)]
pub struct SendCodeRequest {
    pub mobile: String,
}

#[derive(Serialize)]
pub struct SendCode {
    pub found_account: bool,
}

async fn send_code(_body: Json<SendCodeRequest>) -> Json<SendCode> {
    Json(SendCode { found_account: false })
}

#[test]
fn test_basic_actix() {
    let s = Server::new()
        .get("/hello", send_code)
        ;
    println!("{:?}", s.openapi);
    assert_eq!(1, 0);
}