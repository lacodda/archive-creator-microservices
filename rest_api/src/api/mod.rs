use actix_web::web;
use utoipa::OpenApi;
pub mod enqueue;
pub mod get_archive;
pub mod get_progress;
pub mod stop_task;
pub mod task {
    tonic::include_proto!("task");
}

#[derive(OpenApi)]
#[openapi(paths(enqueue::enqueue_archive,), components(schemas(enqueue::ArchiveForm)))]
pub struct Api;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(enqueue::enqueue_archive)
        .service(get_archive::get_archive)
        .service(get_progress::get_progress)
        .service(stop_task::stop_task);
}
