use crate::models::task::Task;
use crate::services::task_service::create_zip_with_password;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::utils::zip::generate_random_password;
use std::collections::HashMap;
use task::task_service_server::TaskService;
use task::{EnqueueTaskRequest, StopTaskRequest, StopTaskResponse, TaskIdResponse, TaskProgressRequest, TaskProgressResponse};

pub mod task {
    tonic::include_proto!("task");
}

#[derive(Default)]
pub struct TaskServiceImpl {
    tasks: Arc<Mutex<HashMap<String, Task>>>,
}

#[tonic::async_trait]
impl TaskService for TaskServiceImpl {
    async fn enqueue_task(&self, request: Request<EnqueueTaskRequest>) -> Result<Response<TaskIdResponse>, Status> {
        let req = request.into_inner();
        let task_id = Uuid::new_v4().to_string();
        let file_path = Self::get_file_path(&task_id);
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
            }))
        } else {
            Err(Status::not_found("Task not found"))
        }
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
}

impl TaskServiceImpl {
    fn get_file_path(task_id: &str) -> String {
        format!("C:/Tmp/{}.zip", task_id)
    }
}
