mod api;
mod services;
mod models;
mod utils;

use tonic::transport::Server;
use api::task_service::task_service_server::TaskServiceServer;
use api::TaskServiceImpl;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let task_service = TaskServiceImpl::default();

    Server::builder()
        .add_service(TaskServiceServer::new(task_service))
        .serve(addr)
        .await?;

    Ok(())
}