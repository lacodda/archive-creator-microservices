use crate::{
    api::task::{EnqueueTaskRequest, FileInfo},
    error::ErrorResponse,
    AppState,
};
use actix_multipart::Multipart;
use actix_web::{post, web, Error, HttpResponse};
use futures::StreamExt;
use tonic::Request;

#[post("/enqueue")]
pub async fn enqueue_archive(mut payload: Multipart, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let mut archive_name = String::new();
    let mut files = Vec::new();

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(f) => f,
            Err(_) => {
                return Ok(HttpResponse::InternalServerError().json(ErrorResponse::new("InternalServerError", "Failed to process multipart field")));
            }
        };
        let content_disposition = field.content_disposition();
        let name = content_disposition.get_name().unwrap();

        if name == "archive_name" {
            while let Some(chunk) = field.next().await {
                let data = chunk?;
                archive_name.push_str(std::str::from_utf8(&data).unwrap());
            }
        } else if name == "files" {
            let filename = content_disposition.get_filename().unwrap().to_string();
            let mut buffer = Vec::new();
            while let Some(chunk) = field.next().await {
                let data = chunk?;
                buffer.extend_from_slice(&data);
            }
            files.push(FileInfo { filename, content: buffer });
        }
    }

    let request = Request::new(EnqueueTaskRequest { archive_name, files });

    let mut client = data.task_client.lock().await;
    match client.enqueue_task(request).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response.into_inner())),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse::from(e))),
    }
}
