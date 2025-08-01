use oasgen::{OaSchema, Server, oasgen};
use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use tracing::error;

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
async fn send_code(_body: Json<SendCode>) -> Result<Json<SendCodeResponse>, (StatusCode, String)> {
    if _body.0.mobile.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Mobile number cannot be empty or invalid".to_string(),
        ));
    }

    // Simulate another error
    if _body.0.mobile == "blocked" {
        return Err((
            StatusCode::FORBIDDEN,
            format!("This account {} is blocked", _body.0.mobile),
        ));
    }

    // New local variable assignment using map_err
    let vault_id_str = "not-an-integer";
    let vault_id: i32 = vault_id_str
        .parse::<i32>()
        .map_err(|_e| {
            (
                StatusCode::NOT_FOUND,
                "Invalid vault ID".to_string(),
            )
        })?;


    let db_result: Result<(), &str> = Err("DB issue");

    db_result
        .map_err(|err| {
            error!(?err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error occurred".to_string(),
            )
        })?;

    Ok(Json(SendCodeResponse { found_account: true }))
}





fn main() {
    use pretty_assertions::assert_eq;
    let server = Server::axum()
        .post("/hello", send_code)
        ;

    let spec = &server.openapi;
    //let other = serde_yaml::from_str::<oasgen::OpenAPI>(include_str!("01-hello.yaml")).unwrap();
    let spec_yaml = serde_yaml::to_string(&spec).unwrap();
    let other_yaml = include_str!("01-hello.yaml");
    assert_eq!(spec_yaml, other_yaml);
    //assert_eq!(spec, &other);
    let router = axum::Router::new()
        .merge(server.freeze().into_router());
    router.into_make_service();
}