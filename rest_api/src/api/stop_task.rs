use actix_web::{get, web, Error, HttpResponse};
use tonic::Request;
use task::task_service_client::TaskServiceClient;
use task::{StopTaskRequest, StopTaskResponse};

pub mod task {
    tonic::include_proto!("task");
}

#[derive(Deserialize)]
pub struct StopTaskQuery {
    taskId: String,
}

#[get("/stop")]
pub async fn stop_task(
    query: web::Query<StopTaskQuery>,
) -> Result<HttpResponse, Error> {
    let mut client = TaskServiceClient::connect("http://[::1]:50051").await?;
    let request = Request::new(StopTaskRequest {
        task_id: query.taskId.clone(),
    });

    let response: StopTaskResponse = client.stop_task(request).await?.into_inner();
    Ok(HttpResponse::Ok().body(response.status))
}
