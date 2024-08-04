use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorResponse {
    code: String,
    message: String,
}

impl ErrorResponse {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_owned(),
            message: message.to_owned(),
        }
    }
}

impl From<tonic::Status> for ErrorResponse {
    fn from(status: tonic::Status) -> Self {
        ErrorResponse {
            code: status.code().to_string(),
            message: status.message().to_string(),
        }
    }
}
