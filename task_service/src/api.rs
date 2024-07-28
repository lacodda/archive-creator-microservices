use tonic::{Request, Response, Status};
use uuid::Uuid;
use tokio::sync::Mutex;
use std::sync::Arc;

use crate::models::task::Task;
use crate::services::task_service::TaskServiceImpl;
use crate::utils::zip::generate_random_password;
use task::task_service_server::TaskService;
use task::{EnqueueTaskRequest, TaskIdResponse, TaskProgressRequest, TaskProgressResponse, StopTaskRequest, StopTaskResponse};

pub mod task {
    tonic::include_proto!("task");
}

#[derive(Default)]
pub struct TaskServiceImpl {
    tasks: Arc<Mutex<HashMap<String, Task>>>,
}

#[tonic::async_trait]
impl TaskService for TaskServiceImpl {
    async fn enqueue_task(
        &self,
        request: Request<EnqueueTaskRequest>,
    ) -> Result<Response<TaskIdResponse>, Status> {
        let req = request.into_inner();
        let task_id = Uuid::new_v4().to_string();
        let password = generate_random_password();
        let archive_name = req.archive_name;
        let files = req.files;

        let stop_signal = Arc::new(AtomicBool::new(false));

        let mut tasks = self.tasks.lock().await;
        tasks.insert(task_id.clone(), Task {
            taskId: task_id.clone(),
            description: None,
            done: false,
            error: None,
            progress: 0.0,
            sourceHash: None,
            timestamp: Utc::now().to_rfc3339(),
            password: password.clone(),
            archive_name: archive_name.clone(),
            stop_signal: stop_signal.clone(),
        });

        // Start the archive creation process in a new async task
        let tasks_clone = self.tasks.clone();
        tokio::spawn(async move {
            let total_steps = 10;
            let step_duration = tokio::time::Duration::from_secs(6); // 10 steps, each 6 seconds = 60 seconds
            for step in 1..=total_steps {
                tokio::time::sleep(step_duration).await;
                if stop_signal.load(std::sync::atomic::Ordering::Relaxed) {
                    let mut tasks = tasks_clone.lock().await;
                    tasks.remove(&task_id);
                    return;
                }

                let progress = (step as f64 / total_steps as f64) * 100.0;
                let mut tasks = tasks_clone.lock().await;
                if let Some(task) = tasks.get_mut(&task_id) {
                    task.progress = progress;
                }
            }

            match crate::services::task_service::create_zip_with_password(
                &format!("/tmp/{}.zip", task_id),
                files,
                &password,
            ) {
                Ok(_) => {
                    let mut tasks = tasks_clone.lock().await;
                    if let Some(task) = tasks.get_mut(&task_id) {
                        task.done = true;
                        task.progress = 100.0;
                    }
                }
                Err(e) => {
                    let mut tasks = tasks_clone.lock().await;
                    if let Some(task) = tasks.get_mut(&task_id) {
                        task.done = false;
                        task.error = Some(format!("Failed to create archive: {:?}", e));
                    }
                }
            }
        });

        Ok(Response::new(TaskIdResponse { task_id }))
    }

    async fn get_task_progress(
        &self,
        request: Request<TaskProgressRequest>,
    ) -> Result<Response<TaskProgressResponse>, Status> {
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

    async fn stop_task(
        &self,
        request: Request<StopTaskRequest>,
    ) -> Result<Response<StopTaskResponse>, Status> {
        let task_id = request.into_inner().task_id;
        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.get_mut(&task_id) {
            task.stop_signal.store(true, std::sync::atomic::Ordering::Relaxed);
            Ok(Response::new(StopTaskResponse { status: "Task stopping".into() }))
        } else {
            Err(Status::not_found("Task not found"))
        }
    }
}
