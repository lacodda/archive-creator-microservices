use actix_web::{get, web, Error, HttpResponse};
use tonic::Request;
use task::task_service_client::TaskServiceClient;
use task::{TaskProgressRequest, TaskProgressResponse};

pub mod task {
    tonic::include_proto!("task");
}

#[derive(Deserialize)]
pub struct GetProgressQuery {
    taskId: Option<String>,
}

#[get("/progress")]
pub async fn get_progress(
    query: web::Query<GetProgressQuery>,
) -> Result<HttpResponse, Error> {
    if let Some(task_id) = &query.taskId {
        let mut client = TaskServiceClient::connect("http://[::1]:50051").await?;
        let request = Request::new(TaskProgressRequest {
            task_id: task_id.clone(),
        });

        let response: TaskProgressResponse = client.get_task_progress(request).await?.into_inner();
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::BadRequest().body("taskId query parameter is required"))
    }
}
