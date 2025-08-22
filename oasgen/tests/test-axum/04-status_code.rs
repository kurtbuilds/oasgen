use oasgen::{OaSchema, Server, oasgen};
use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;  


#[derive(Deserialize, OaSchema)]
pub struct Task {
    pub title: String,
    pub priority: Option<u8>,
}

#[derive(Serialize, OaSchema)]
pub struct TaskResponse {
    pub id: u32,
    pub message: String,
}

#[oasgen(tags("tasks"), summary = "Create a new task with error cases")]
async fn create_task(Json(task): Json<Task>) -> Result<Json<TaskResponse>, (StatusCode, String)> {

    if task.title.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Task title cannot be empty".to_string(),
        ));
    }

    if let Some(priority) = task.priority {
        if priority > 5 {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "Priority must be between 1 and 5".to_string(),
            ));
        }
    }

    let db_result: Result<u32, &str> = Err("DB error");

    let id: u32 = db_result.map_err(|_err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to save task".to_string(),
        )
    })?;

    Ok(Json(TaskResponse {
        id,
        message: format!("Task '{}' created successfully", task.title),
    }))
}



fn main() {
    use pretty_assertions::assert_eq;

    let server: Server<_> = Server::axum()
        .post("/tasks", create_task)
        ;

    let spec_yaml = serde_yaml::to_string(&server.openapi).unwrap();
    let expected_yaml = include_str!("04-status_code.yaml");

    let spec_value: Value = serde_yaml::from_str(&spec_yaml).unwrap();
    let expected_value: Value = serde_yaml::from_str(expected_yaml).unwrap();

    assert_eq!(spec_value, expected_value);
    let router = axum::Router::new()
        .merge(server.freeze().into_router());
    router.into_make_service();
}
