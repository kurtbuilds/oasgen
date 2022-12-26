use oasgen::{OaSchema, Server};
use actix_web::web::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, OaSchema)]
pub struct SendCodeRequest {
    pub mobile: String,
}

#[derive(Serialize, OaSchema)]
pub struct SendCode {
    pub found_account: bool,
}

async fn send_code(_body: Json<SendCodeRequest>) -> Json<SendCode> {
    Json(SendCode { found_account: false })
}

pub struct ResponseWrapper<T>(pub T);

pub struct TypeEncoder<Args, Response, Future>(Future, std::marker::PhantomData<(Args, Response)>);


#[test]
fn test_basic_actix() {
    use std::fs::File;
    let s = Server::new()
        .get("/hello", send_code)
        ;
    serde_yaml::to_writer(&File::create("tests/01-hello.yaml").unwrap(), &s.openapi).unwrap();
    println!("{:?}", s.openapi);
    assert_eq!(1, 0);
}