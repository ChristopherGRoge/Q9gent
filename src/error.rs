use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Process spawn failed: {0}")]
    ProcessSpawnFailed(String),

    #[error("Process execution error: {0}")]
    ProcessExecutionError(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::ProcessSpawnFailed(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::ProcessExecutionError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::SessionNotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::IoError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            AppError::SerializationError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
