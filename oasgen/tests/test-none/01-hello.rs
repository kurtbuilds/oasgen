use oasgen::{OaSchema, Server, oasgen};
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
async fn send_code(_body: SendCode) -> SendCodeResponse {
    SendCodeResponse { found_account: false }
}

#[allow(unused)]
#[oasgen]
async fn no_params() -> SendCodeResponse {
    SendCodeResponse { found_account: false }
}

fn main() {
    let _ = Server::none()
        .get("/hello", send_code)
        // .get("/no_params", no_params())
        ;
}