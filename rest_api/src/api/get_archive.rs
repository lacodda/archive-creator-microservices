use actix_web::{get, web, Error, HttpResponse};
use std::fs::File;
use std::io::Read;

#[get("/archive")]
pub async fn get_archive(task_id: web::Query<String>) -> Result<HttpResponse, Error> {
    let file_path = format!("/tmp/{}.zip", task_id);

    let mut buffer = Vec::new();
    let mut file = File::open(file_path)?;
    file.read_to_end(&mut buffer)?;

    Ok(HttpResponse::Ok()
        .content_type("application/x-zip-compressed")
        .insert_header(("Content-Disposition", format!("attachment; filename=\"{}.zip\"", task_id)))
        .body(buffer))
}
