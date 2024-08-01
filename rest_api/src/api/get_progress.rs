use crate::{
    api::task::{TaskProgressRequest, TaskProgressResponse as ProtoTaskProgressResponse},
    AppState,
};
use actix_web::{get, web, Error, HttpResponse};
use serde::{Deserialize, Serialize};
use tonic::Request;

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct GetProgressQuery {
    taskId: Option<String>,
}

#[derive(Serialize)]
struct TaskProgressResponse {
    task_id: String,
    done: bool,
    progress: f64,
    error: String,
}

impl From<ProtoTaskProgressResponse> for TaskProgressResponse {
    fn from(item: ProtoTaskProgressResponse) -> Self {
        TaskProgressResponse {
            task_id: item.task_id,
            done: item.done,
            progress: item.progress,
            error: item.error,
        }
    }
}

#[get("/progress")]
pub async fn get_progress(query: web::Query<GetProgressQuery>, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    if let Some(task_id) = &query.taskId {
        let request = Request::new(TaskProgressRequest { task_id: task_id.clone() });
        let mut client = data.task_client.lock().await;
        let response = client.get_task_progress(request).await;

        match response {
            Ok(res) => {
                let proto_item = res.into_inner();
                let item: TaskProgressResponse = proto_item.into();
                Ok(HttpResponse::Ok().json(item))
            }
            Err(_) => Ok(HttpResponse::InternalServerError().body("Internal Server Error")),
        }
    } else {
        Ok(HttpResponse::BadRequest().body("taskId query parameter is required"))
    }
}
