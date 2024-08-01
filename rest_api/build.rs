fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .type_attribute("task.FileInfo", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute("task.TaskIdResponse", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute("task.TaskProgressRequest", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute("task.TaskProgressResponse", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute("task.AllTasksRequest", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute("task.AllTasksResponse", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute("task.StopTaskRequest", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute("task.StopTaskResponse", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile(&["../proto/task_service.proto"], &["../proto"])?;
    Ok(())
}
