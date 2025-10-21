use std::error::Error;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Error as AxumError, Json, body};

use serde_json::{Error as SerdeError, Value, json};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MiddlewareError {
    #[error("Failed to read request body")]
    ReadFailed { error: AxumError },

    #[error("Body size exceeds limit")]
    BodySizeExceeded,

    #[error("Failed to parse request body")]
    ParseFailed { error: SerdeError },

    #[error("Validation error")]
    ValidationFailed { error: Vec<String> },
}

impl MiddlewareError {
    pub fn status_code(&self) -> (StatusCode, String) {
        match self {
            MiddlewareError::ReadFailed { error } => {
                (StatusCode::BAD_REQUEST, "CRF0001".to_string())
            }
            MiddlewareError::BodySizeExceeded => (StatusCode::BAD_REQUEST, "CRF0002".to_string()),
            MiddlewareError::ParseFailed { error } => {
                (StatusCode::BAD_REQUEST, "CRF0003".to_string())
            }
            MiddlewareError::ValidationFailed { error } => {
                (StatusCode::BAD_REQUEST, "CRF0004".to_string())
            }
        }
    }
    fn get_response_json(&self) -> Json<Value> {
        let (status, code) = self.status_code();
        let error_string = self.to_string();
        let source_error_string = self
            .source()
            .map(|e| e.to_string())
            .unwrap_or_else(|| "No additional error information".to_string());
        Json(json!({
            "error": error_string,
            "source_error": source_error_string,
            "code": code,
            "status": status.as_u16(),
        }))
    }
}

impl IntoResponse for MiddlewareError {
    fn into_response(self) -> Response {
        let (status, code) = self.status_code();
        let body = self.get_response_json();
        (status, body).into_response()
    }
}
