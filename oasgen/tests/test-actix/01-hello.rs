use oasgen::{OaSchema, Server, oasgen};
use actix_web::web::{Json, Query};
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

#[derive(Deserialize, OaSchema)]
pub struct GetCode {
    pub code: String,
}

#[derive(Serialize, OaSchema)]
pub struct CodeResponse {
    pub found_code: bool,
}

#[oasgen]
async fn get_code(Query(GetCode { code }): Query<GetCode>) -> Json<CodeResponse> {
    Json(CodeResponse {
        found_code: matches!(&*code, "1234" | "5678"),
    })
}

fn main() {
    use pretty_assertions::assert_eq;
    let server = Server::actix()
        .post("/hello", send_code)
        .get("/get-code", get_code)
        ;
    let spec = serde_yaml::to_string(&server.openapi).unwrap();
    println!("{}", spec);
    let other = include_str!("01-hello.yaml");
    assert_eq!(spec.trim(), other);
}