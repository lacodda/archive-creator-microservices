use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::{atomic::AtomicBool, Arc};

#[derive(Serialize, Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct Task {
    pub taskId: String,
    pub description: Option<String>,
    pub done: bool,
    pub error: Option<String>,
    pub progress: f64,
    pub sourceHash: Option<String>,
    pub timestamp: String,
    pub password: String,
    pub archive_name: String,
    #[serde(skip)] // Skip this field during serialization and deserialization
    pub stop_signal: Arc<AtomicBool>,
}

impl Task {
    pub fn new(task_id: &str, archive_name: &str, password: &str) -> Self {
        Task {
            taskId: task_id.to_owned(),
            description: None,
            done: false,
            error: None,
            progress: 0.0,
            sourceHash: None,
            timestamp: Utc::now().to_rfc3339(),
            password: password.to_owned(),
            archive_name: archive_name.to_owned(),
            stop_signal: Arc::new(AtomicBool::new(false)),
        }
    }
}
