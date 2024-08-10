use actix_web::{web, App, HttpServer};
use api::task::task_service_client::TaskServiceClient;
use config::{Config, Environment, File};
use serde::Deserialize;
use std::{error::Error, io, sync::Arc};
use tokio::sync::Mutex;
use tonic::transport::Channel;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod api;
mod error;

#[derive(Deserialize)]
struct TaskServiceConfig {
    address: String,
    protocol: String,
}

#[derive(Deserialize)]
struct RestApiConfig {
    address: String,
    protocol: String,
}

#[derive(Clone)]
struct AppState {
    task_client: Arc<Mutex<TaskServiceClient<Channel>>>,
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    #[derive(OpenApi)]
    #[openapi(
        info(
            title = "Archive Creator API",
            description = "API for creating password-protected archives from uploaded files. \
            This service supports asynchronous processing and provides endpoints for enqueuing tasks, \
            checking progress, retrieving archives, and stopping tasks.",
            version = "0.0.1",
            contact(
                name = "API Support",
                url = "https://www.example.com/support",
                email = "support@example.com"
            ),
            license(
                name = "MIT",
                url = "https://opensource.org/licenses/MIT"
            )
        ),
        
        paths(
            api::enqueue::enqueue_archive,
            api::get_archive::get_archive,
            api::get_progress::get_progress,
            api::stop_task::stop_task,
        ),
        components(schemas(
            api::enqueue::ArchiveForm,
            api::get_progress::TaskProgressResponse,
            api::get_progress::SingleTaskResponse,
            api::get_progress::AllTasksResponse,
            api::stop_task::StopTaskResponse,
            api::enqueue::TaskIdResponse,
            error::ErrorResponse
        )),
        tags(
            (name = "archive", description = "Archive management endpoints"),
        ),
    )]
    struct ApiDoc;

    let builder = Config::builder();
    let settings = builder
        .add_source(File::with_name("config"))
        .add_source(Environment::with_prefix("APP"))
        .build()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let rest_api_config: RestApiConfig = settings.get("rest_api").map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let task_service_config: TaskServiceConfig = settings.get("task_service").map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    // Initialize gRPC clients
    let task_client = TaskServiceClient::connect(format!("{}://{}", &task_service_config.protocol, &task_service_config.address))
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let app_state = AppState {
        task_client: Arc::new(Mutex::new(task_client)),
    };
    let openapi = ApiDoc::openapi();

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            .service(web::scope("/api/v1").configure(api::init_routes))
            .service(SwaggerUi::new("/swagger/{_:.*}").url("/api/docs/openapi.json", openapi.clone()))
    })
    .bind(&rest_api_config.address)?;

    println!("Server running at {}://{}", &rest_api_config.protocol, &rest_api_config.address);

    server.run().await?;

    Ok(())
}
