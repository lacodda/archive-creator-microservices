use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
#[schema(description = "Error response returned by the API when a request fails")]
pub struct ErrorResponse {
    /// Error code representing the type of error
    code: String,
    /// Human-readable error message
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
