use axum::extract::{Path, Json};
use oasgen::{OaSchema, oasgen, Server};
use serde::{Deserialize};

/// Send a code to a mobile number
#[derive(Deserialize, OaSchema)]
pub struct TaskFilter {
    pub completed: bool,
    pub assigned_to: i32,
}

#[oasgen]
async fn get_task(Path(_id): Path<u64>) -> Json<()> {
    Json(())
}

#[oasgen]
async fn get_stuff(Path((_id, _tu)): Path<(u64, u64)>) -> Json<()> {
    Json(())
}

fn main() {
    use pretty_assertions::assert_eq;
    let server = Server::axum()
        .get("/tasks/{id}/", get_task)
        .get("/tasks/{id}/{tu}", get_stuff)
        ;

    let spec = serde_yaml::to_string(&server.openapi).unwrap();
    let other = include_str!("03-path.yaml");
    assert_eq!(spec.trim(), other);
    let router = axum::Router::new()
        .merge(server.freeze().into_router());
    router.into_make_service();
}