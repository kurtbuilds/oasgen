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

    let spec = &server.openapi;
    let other = serde_yaml::from_str::<oasgen::OpenAPI>(include_str!("02-path.yaml")).unwrap();
    assert_eq!(spec, &other);
    let router = axum::Router::new()
        .merge(server.freeze().into_router());
    router.into_make_service();
}