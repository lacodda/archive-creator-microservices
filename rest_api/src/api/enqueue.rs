use actix_multipart::Multipart;
use actix_web::{post, web, Error, HttpResponse};
use futures::StreamExt;
use tonic::Request;
use task::task_service_client::TaskServiceClient;
use task::{EnqueueTaskRequest, TaskIdResponse};

pub mod task {
    tonic::include_proto!("task");
}

#[post("/enqueue")]
pub async fn enqueue_archive(
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    let mut archive_name = String::new();
    let mut files = Vec::new();

    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_disposition = field.content_disposition().unwrap();
        let name = content_disposition.get_name().unwrap();

        if name == "archive_name" {
            while let Some(chunk) = field.try_next().await? {
                archive_name.push_str(std::str::from_utf8(&chunk).unwrap());
            }
        } else if name == "files" {
            let mut buffer = Vec::new();
            while let Some(chunk) = field.next().await {
                let data = chunk?;
                buffer.extend_from_slice(&data);
            }
            files.push(buffer);
        }
    }

    let mut client = TaskServiceClient::connect("http://[::1]:50051").await?;
    let request = Request::new(EnqueueTaskRequest {
        archive_name,
        files,
    });

    let response = client.enqueue_task(request).await?;

    Ok(HttpResponse::Ok().json(response.into_inner()))
}
