use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Config(String),
    NotFound(String),
    BadRequest(String),
    ProviderError { provider: String, message: String },
    Internal(String),
}

impl std::error::Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Config(msg) => write!(f, "Configuration error: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::ProviderError { provider, message } => {
                write!(f, "Provider '{}' error: {}", provider, message)
            }
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match self {
            AppError::Config(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "server_error",
                format!("Config error: {}", msg),
            ),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found_error", msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "invalid_request_error", msg),
            AppError::ProviderError { provider, message } => (
                StatusCode::BAD_GATEWAY,
                "provider_error",
                format!("Provider error ({}): {}", provider, message),
            ),
            AppError::Internal(msg) => {
                tracing::error!("Internal error encountered: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "server_error",
                    "An internal server error occurred".to_string(),
                )
            }
        };

        let body = Json(ErrorResponse {
            error: ErrorDetail {
                message,
                error_type: error_type.to_string(),
                param: None,
                code: None,
            },
        });

        (status, body).into_response()
    }
}
