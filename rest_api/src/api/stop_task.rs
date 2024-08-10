use crate::{
    api::task::{StopTaskRequest, StopTaskResponse as ProtoStopTaskResponse},
    error::ErrorResponse,
    AppState,
};
use actix_web::{get, web, Error, HttpResponse};
use serde::Deserialize;
use tonic::Request;
use utoipa::ToSchema;

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct StopTaskQuery {
    /// Task ID of the task to be stopped
    taskId: String,
}

#[derive(ToSchema)]
#[allow(unused)]
#[schema(description = "Response indicating the status of a stopped task")]
pub struct StopTaskResponse {
    /// The status message indicating the result of the stop operation
    status: String,
}

/// Stop a task.
///
/// This endpoint stops a task by its ID. If the task is found and stopped successfully,
/// it returns a status message.
///
/// Example of a successful response:
/// ```json
/// {
///     "status": "Task stopping"
/// }
/// ```
#[utoipa::path(
    path = "/api/v1/stop",
    params(
        ("taskId" = String, description = "Task ID of the task to be stopped")
    ),
    responses(
        (status = 200, description = "Task stopped successfully", body = StopTaskResponse),
        (status = 404, description = "Task not found", body = ErrorResponse)
    )
)]
#[get("/stop")]
pub async fn stop_task(query: web::Query<StopTaskQuery>, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let mut client = data.task_client.lock().await;
    let request = Request::new(StopTaskRequest { task_id: query.taskId.clone() });
    let response = client.stop_task(request).await;

    match response {
        Ok(res) => {
            let proto_item = res.into_inner();
            let item: ProtoStopTaskResponse = proto_item.into();
            Ok(HttpResponse::Ok().json(item))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse::from(e))),
    }
}
