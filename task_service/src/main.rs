use api::task::task_service_server::TaskServiceServer;
use api::TaskServiceImpl;
use common::parse_size;
use config::{Config, Environment, File};
use serde::Deserialize;
use std::io;
use tonic::transport::Server;

mod api;
mod models;
mod services;
mod utils;

#[derive(Deserialize)]
struct TaskServiceConfig {
    address: String,
    max_message_size: String,
    archive_path: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let builder = Config::builder();
    let settings = builder
        .add_source(File::with_name("config"))
        .add_source(Environment::with_prefix("APP"))
        .build()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let task_service_config: TaskServiceConfig = settings.get("task_service").map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let addr = task_service_config.address.parse()?;
    // Parse max message size
    let max_message_size = parse_size(&task_service_config.max_message_size)?;
    let task_service = TaskServiceImpl::new(&task_service_config.archive_path);

    Server::builder()
        .add_service(
            TaskServiceServer::new(task_service)
                .max_decoding_message_size(max_message_size)
                .max_encoding_message_size(max_message_size),
        )
        .serve(addr)
        .await?;

    Ok(())
}
