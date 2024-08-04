use crate::{
    api::task::{ArchiveResponse, GetArchiveRequest},
    error::ErrorResponse,
    AppState,
};
use actix_web::{get, web, Error, HttpResponse};
use serde::Deserialize;
use tonic::Request;

#[derive(Deserialize)]
#[allow(non_snake_case)]
pub struct GetArchiveQuery {
    taskId: String,
}

#[get("/archive")]
pub async fn get_archive(query: web::Query<GetArchiveQuery>, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let request = Request::new(GetArchiveRequest { task_id: query.taskId.clone() });
    let mut client = data.task_client.lock().await;
    match client.get_archive(request).await {
        Ok(response) => {
            let archive_response: ArchiveResponse = response.into_inner();
            Ok(HttpResponse::Ok()
            .content_type("application/x-zip-compressed")
            .insert_header((
                "Content-Disposition",
                format!("attachment; filename=\"{}\".zip", archive_response.archive_name),
            ))
            .body(archive_response.archive))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse::from(e))),
    }
}
