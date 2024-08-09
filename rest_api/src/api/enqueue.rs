use crate::{
    api::task::{EnqueueTaskRequest, FileInfo},
    error::ErrorResponse,
    AppState,
};
use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{post, web, Error, HttpResponse};
use std::io::Read;
use tonic::Request;
use utoipa::ToSchema;

#[derive(Debug, MultipartForm, ToSchema)]
pub struct ArchiveForm {
    #[schema(value_type = String)]
    archive_name: Text<String>,
    #[schema(value_type = Vec<String>, format = Binary)]
    files: Vec<TempFile>,
}

#[utoipa::path(
    post,
    request_body(content = ArchiveForm, content_type = "multipart/form-data"),
    responses((status = 200))
)]
#[post("/enqueue")]
pub async fn enqueue_archive(MultipartForm(form): MultipartForm<ArchiveForm>, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let archive_name = form.archive_name.clone();
    let mut files = Vec::new();

    for file in form.files.iter() {
        let filename = file.file_name.clone().unwrap_or_else(|| "unknown".to_string());
        let mut buffer = Vec::new();
        if let Err(_e) = std::fs::File::open(&file.file).and_then(|mut f| f.read_to_end(&mut buffer)) {
            return Ok(HttpResponse::InternalServerError().json(ErrorResponse::new("InternalServerError", &format!("Failed to read file {}", filename))));
        }
        files.push(FileInfo { filename, content: buffer });
    }

    let request = Request::new(EnqueueTaskRequest { archive_name, files });

    let mut client = data.task_client.lock().await;
    match client.enqueue_task(request).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response.into_inner())),
        Err(e) => Ok(HttpResponse::InternalServerError().json(ErrorResponse::from(e))),
    }
}
