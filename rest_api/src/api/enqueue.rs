use crate::{
    api::task::{EnqueueTaskRequest, FileInfo, TaskIdResponse as ProtoTaskIdResponse},
    AppState,
};
use actix_multipart::Multipart;
use actix_web::{post, web, Error, HttpResponse};
use futures::StreamExt;
use serde::Serialize;
use tonic::Request;

#[derive(Serialize)]
struct TaskIdResponse {
    task_id: String,
}

impl From<ProtoTaskIdResponse> for TaskIdResponse {
    fn from(item: ProtoTaskIdResponse) -> Self {
        TaskIdResponse { task_id: item.task_id }
    }
}

#[post("/enqueue")]
pub async fn enqueue_archive(mut payload: Multipart, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let mut archive_name = String::new();
    let mut files = Vec::new();

    while let Some(item) = payload.next().await {
        let mut field = item?;
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
    let response = client.enqueue_task(request).await;

    match response {
        Ok(res) => {
            let proto_item = res.into_inner();
            let item: TaskIdResponse = proto_item.into();
            Ok(HttpResponse::Ok().json(item))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().body("Internal Server Error")),
    }
}
