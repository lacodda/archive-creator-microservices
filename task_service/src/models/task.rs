use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::{atomic::AtomicBool, Arc};

#[derive(Serialize, Deserialize, Clone)]
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
    pub stop_signal: Arc<AtomicBool>,
}
