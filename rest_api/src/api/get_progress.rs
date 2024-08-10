use super::task::task_service_client::TaskServiceClient;
use crate::{
    api::task::{AllTasksRequest, AllTasksResponse as ProtoAllTasksResponse, TaskProgressRequest, TaskProgressResponse as ProtoTaskProgressResponse},
    error::ErrorResponse,
    AppState,
};
use actix_web::{get, web, Error, HttpResponse};
use serde::Deserialize;
use tonic::{transport::Channel, Request, Status};
use utoipa::ToSchema;

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct GetProgressQuery {
    /// Optional Task ID to check the progress of a specific task
    taskId: Option<String>,
}

#[derive(ToSchema)]
#[allow(unused)]
#[schema(description = "Response containing the progress details of a task")]
pub struct SingleTaskResponse {
    /// The unique identifier of the task
    task_id: String,
    /// Indicates whether the task is completed
    done: bool,
    /// The progress of the task as a percentage
    progress: i16,
    /// An error message if the task failed
    error: Option<String>,
}

#[derive(ToSchema)]
#[allow(unused)]
#[schema(description = "Response containing a list of all tasks and their progress")]
pub struct AllTasksResponse {
    /// List of tasks with their progress details
    tasks: Vec<SingleTaskResponse>,
}

#[derive(ToSchema)]
#[allow(unused)]
pub enum TaskProgressResponse {
    Single(SingleTaskResponse),
    All(AllTasksResponse)
}

async fn fetch_task_progress(task_id: String, client: &mut TaskServiceClient<Channel>) -> Result<ProtoTaskProgressResponse, Status> {
    let request = Request::new(TaskProgressRequest { task_id });
    let response = client.get_task_progress(request).await?;
    Ok(response.into_inner())
}

async fn fetch_all_tasks(client: &mut TaskServiceClient<Channel>) -> Result<ProtoAllTasksResponse, Status> {
    let request = Request::new(AllTasksRequest {});
    let response = client.get_all_tasks(request).await?;
    Ok(response.into_inner())
}

/// Check the progress of a task or retrieve all tasks.
///
/// This endpoint allows you to check the progress of a specific task by its ID,
/// or to retrieve the list of all tasks and their progress. If a `taskId` is provided,
/// it returns the progress of that specific task. If no `taskId` is provided,
/// it returns the progress of all tasks.
///
/// Example of a successful response when `taskId` is provided:
/// ```json
/// {
///     "task_id": "123e4567-e89b-12d3-a456-426614174000",
///     "done": false,
///     "progress": 50,
///     "error": null
/// }
/// ```
///
/// Example of a successful response when `taskId` is not provided:
/// ```json
/// {
///     "tasks": [
///         {
///             "task_id": "123e4567-e89b-12d3-a456-426614174000",
///             "done": false,
///             "progress": 50,
///             "error": null
///         },
///         {
///             "task_id": "223e4567-e89b-12d3-a456-426614174001",
///             "done": true,
///             "progress": 100,
///             "error": null
///         }
///     ]
/// }
/// ```
#[utoipa::path(
    path = "/api/v1/progress",
    params(
        ("taskId" = Option<String>, description = "Optional Task ID to check the progress of a specific task")
    ),
    responses(
        (status = 200, description = "Progress retrieved successfully", body = TaskProgressResponse, content_type = "application/json"),
        (status = 404, description = "Task not found", body = ErrorResponse)
    )
)]
#[get("/progress")]
pub async fn get_progress(query: web::Query<GetProgressQuery>, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let mut client = data.task_client.lock().await;

    if let Some(task_id) = &query.taskId {
        match fetch_task_progress(task_id.clone(), &mut client).await {
            Ok(progress_response) => Ok(HttpResponse::Ok().json(progress_response)),
            Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse::from(e))),
        }
    } else {
        match fetch_all_tasks(&mut client).await {
            Ok(all_tasks_response) => Ok(HttpResponse::Ok().json(all_tasks_response.tasks)),
            Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse::from(e))),
        }
    }
}
