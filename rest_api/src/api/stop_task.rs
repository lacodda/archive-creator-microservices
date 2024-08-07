use crate::{
    api::task::{StopTaskRequest, StopTaskResponse},
    error::ErrorResponse,
    AppState,
};
use actix_web::{get, web, Error, HttpResponse};
use serde::Deserialize;
use tonic::Request;

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct StopTaskQuery {
    taskId: String,
}

#[get("/stop")]
pub async fn stop_task(query: web::Query<StopTaskQuery>, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let mut client = data.task_client.lock().await;
    let request = Request::new(StopTaskRequest { task_id: query.taskId.clone() });
    let response = client.stop_task(request).await;

    match response {
        Ok(res) => {
            let proto_item = res.into_inner();
            let item: StopTaskResponse = proto_item.into();
            Ok(HttpResponse::Ok().json(item))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse::from(e))),
    }
}
