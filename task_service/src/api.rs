use crate::models::task::Task;
use crate::services::task_service::create_zip_with_password;
use crate::utils::zip::generate_random_password;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use task::task_service_server::TaskService;
use task::{
    AllTasksRequest, AllTasksResponse, ArchiveResponse, EnqueueTaskRequest, GetArchiveRequest, StopTaskRequest, StopTaskResponse, TaskIdResponse,
    TaskProgressRequest, TaskProgressResponse,
};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub mod task {
    tonic::include_proto!("task");
}

#[derive(Default)]
pub struct TaskServiceImpl {
    tasks: Arc<Mutex<HashMap<String, Task>>>,
    archive_path: String,
}

impl TaskServiceImpl {
    pub fn new(archive_path: &str) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            archive_path: archive_path.to_owned(),
        }
    }

    pub fn get_file_path(&self, task_id: &str) -> String {
        format!("{}/{}.zip", self.archive_path, task_id)
    }
}

#[tonic::async_trait]
impl TaskService for TaskServiceImpl {
    async fn enqueue_task(&self, request: Request<EnqueueTaskRequest>) -> Result<Response<TaskIdResponse>, Status> {
        let req = request.into_inner();
        let task_id = Uuid::new_v4().to_string();
        let file_path = self.get_file_path(&task_id);
        let password = generate_random_password();
        let archive_name = req.archive_name;
        let files = req.files;

        let task = Task::new(&task_id, &archive_name, &password);
        let stop_signal = task.stop_signal.clone();
        let mut tasks = self.tasks.lock().await;
        tasks.insert(task_id.clone(), task);

        // Start the archive creation process in a new async task
        let task_id_clone = task_id.clone();
        let tasks_clone = self.tasks.clone();
        tokio::spawn(async move {
            let total_steps = 10;
            let step_duration = Duration::from_secs(6); // 10 steps, each 6 seconds = 60 seconds
            for step in 1..=total_steps {
                sleep(step_duration).await;
                if stop_signal.load(Ordering::Relaxed) {
                    let mut tasks = tasks_clone.lock().await;
                    tasks.remove(&task_id_clone);
                    return;
                }

                let progress = (step as f64 / total_steps as f64) * 100.0;

                let mut tasks = tasks_clone.lock().await;
                if let Some(task) = tasks.get_mut(&task_id_clone) {
                    task.progress = progress;
                }
            }

            match create_zip_with_password(&file_path, files, &password) {
                Ok(_) => {
                    let mut tasks = tasks_clone.lock().await;
                    if let Some(task) = tasks.get_mut(&task_id_clone) {
                        task.done = true;
                        task.progress = 100.0;
                    }
                }
                Err(e) => {
                    let mut tasks = tasks_clone.lock().await;
                    if let Some(task) = tasks.get_mut(&task_id_clone) {
                        task.done = false;
                        task.error = Some(format!("Failed to create archive: {:?}", e));
                    }
                }
            }
        });

        Ok(Response::new(TaskIdResponse { task_id }))
    }

    async fn get_task_progress(&self, request: Request<TaskProgressRequest>) -> Result<Response<TaskProgressResponse>, Status> {
        let task_id = request.into_inner().task_id;
        let tasks = self.tasks.lock().await;
        if let Some(task) = tasks.get(&task_id) {
            Ok(Response::new(TaskProgressResponse {
                task_id: task.taskId.clone(),
                done: task.done,
                progress: task.progress,
                error: task.error.clone().unwrap_or_default(),
                timestamp: task.timestamp.clone(),
                password: task.password.clone(),
                archive_name: task.archive_name.clone(),
            }))
        } else {
            Err(Status::not_found("Task not found"))
        }
    }

    async fn get_all_tasks(&self, _request: Request<AllTasksRequest>) -> Result<Response<AllTasksResponse>, Status> {
        let tasks = self.tasks.lock().await;
        let tasks_list: Vec<TaskProgressResponse> = tasks
            .values()
            .map(|task| TaskProgressResponse {
                task_id: task.taskId.clone(),
                done: task.done,
                progress: task.progress,
                error: task.error.clone().unwrap_or_default(),
                timestamp: task.timestamp.clone(),
                password: task.password.clone(),
                archive_name: task.archive_name.clone(),
            })
            .collect();

        Ok(Response::new(AllTasksResponse { tasks: tasks_list }))
    }

    async fn stop_task(&self, request: Request<StopTaskRequest>) -> Result<Response<StopTaskResponse>, Status> {
        let task_id = request.into_inner().task_id;
        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.get_mut(&task_id) {
            task.stop_signal.store(true, std::sync::atomic::Ordering::Relaxed);
            Ok(Response::new(StopTaskResponse {
                status: "Task stopping".into(),
            }))
        } else {
            Err(Status::not_found("Task not found"))
        }
    }

    async fn get_archive(&self, request: Request<GetArchiveRequest>) -> Result<Response<ArchiveResponse>, Status> {
        let task_id = request.into_inner().task_id;
        let tasks = self.tasks.lock().await;
        if let Some(task) = tasks.get(&task_id) {
            let file_path = self.get_file_path(&task_id);
            let mut file: File = File::open(&file_path).map_err(|e| Status::not_found(format!("File not found: {:?}", e)))?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map_err(|e| Status::internal(format!("Failed to read file: {:?}", e)))?;
            Ok(Response::new(ArchiveResponse {
                archive: buffer,
                archive_name: task.archive_name.to_owned(),
            }))
        } else {
            Err(Status::not_found("Task not found"))
        }
    }
}
