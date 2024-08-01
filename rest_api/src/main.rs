use actix_web::{web, App, HttpServer};
use api::task::task_service_client::TaskServiceClient;
use config::{Config, Environment, File};
use serde::Deserialize;
use std::{error::Error, io, sync::Arc};
use tokio::sync::Mutex;
use tonic::transport::Channel;

mod api;

#[derive(Deserialize)]
struct TaskServiceConfig {
    address: String,
}

#[derive(Clone)]
struct AppState {
    task_client: Arc<Mutex<TaskServiceClient<Channel>>>,
}

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let builder = Config::builder();

    let settings = builder
        .add_source(File::with_name("config"))
        .add_source(Environment::with_prefix("APP"))
        .build()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let task_service_config: TaskServiceConfig = settings.get("task_service").map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    // Initialize gRPC clients
    let task_client = TaskServiceClient::connect(task_service_config.address.clone())
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let app_state = AppState {
        task_client: Arc::new(Mutex::new(task_client)),
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            .configure(api::init_routes)
    })
    .bind("127.0.0.1:9188")?
    .run()
    .await
    .map_err(|e| Box::new(e) as Box<dyn Error>)
}
