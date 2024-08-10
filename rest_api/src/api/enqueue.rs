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
    /// The name of the archive to be created
    #[schema(value_type = String)]
    archive_name: Text<String>,
    /// List of files to be included in the archive
    #[schema(value_type = Vec<String>, format = Binary)]
    files: Vec<TempFile>,
}

#[derive(ToSchema)]
#[allow(unused)]
#[schema(description = "Response containing the ID of the enqueued task")]
pub struct TaskIdResponse {
    /// The unique identifier of the task
    task_id: String,
}

/// Enqueue an archive creation task.
///
/// This endpoint enqueues a task to create an archive from the provided files.
/// It accepts a multipart form containing the archive name and the files.
///
/// Returns a task ID which can be used to check the progress of the task.
///
/// Example of a successful response:
/// ```json
/// {
///     "task_id": "123e4567-e89b-12d3-a456-426614174000"
/// }
/// ```
#[utoipa::path(
    path = "/api/v1/enqueue",
    request_body(content = ArchiveForm, content_type = "multipart/form-data", description = "Form data containing the archive name and files"),
    responses(
        (status = 200, description = "Task enqueued successfully", body = TaskIdResponse),
        (status = 500, description = "Failed to enqueue task", body = ErrorResponse)
    )
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
