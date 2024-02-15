use axum::extract::{Query, Json};
use oasgen::{OaSchema, oasgen, Server};
use serde::{Deserialize};

/// Send a code to a mobile number
#[derive(Deserialize, OaSchema)]
pub struct TaskFilter {
    pub completed: bool,
    pub assigned_to: i32,
}

#[oasgen]
async fn list_tasks(Query(_filter): Query<TaskFilter>) -> Json<()> {
    Json(())
}

fn main() {
    use pretty_assertions::assert_eq;
    let server = Server::axum()
        .get("/tasks", list_tasks)
        ;

    let spec = serde_yaml::to_string(&server.openapi).unwrap();
    let other = include_str!("02-query.yaml");
    assert_eq!(spec.trim(), other);
    let router = axum::Router::new()
        .merge(server.freeze().into_router());
    router.into_make_service();
}