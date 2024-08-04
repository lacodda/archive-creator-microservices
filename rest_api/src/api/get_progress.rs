use super::task::task_service_client::TaskServiceClient;
use crate::{
    api::task::{AllTasksRequest, AllTasksResponse, TaskProgressRequest, TaskProgressResponse},
    error::ErrorResponse,
    AppState,
};
use actix_web::{get, web, Error, HttpResponse};
use serde::Deserialize;
use tonic::{transport::Channel, Request, Status};

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct GetProgressQuery {
    taskId: Option<String>,
}

async fn fetch_task_progress(task_id: String, client: &mut TaskServiceClient<Channel>) -> Result<TaskProgressResponse, Status> {
    let request = Request::new(TaskProgressRequest { task_id });
    let response = client.get_task_progress(request).await?;
    Ok(response.into_inner())
}

async fn fetch_all_tasks(client: &mut TaskServiceClient<Channel>) -> Result<AllTasksResponse, Status> {
    let request = Request::new(AllTasksRequest {});
    let response = client.get_all_tasks(request).await?;
    Ok(response.into_inner())
}

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
