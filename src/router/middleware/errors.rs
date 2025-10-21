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
    pub fn get_response_json(&self) -> Json<Value> {
        match self {
            MiddlewareError::ReadFailed { error } => Json(json!({
                "error": self.to_string(),
                "cause": error.to_string(),
                "code": "CRF0001".to_string(),
                "status": StatusCode::BAD_REQUEST.as_u16(),
            })),
            MiddlewareError::BodySizeExceeded => Json(json!({
                "error": self.to_string(),
                "cause": "Request body size exceeds the allowed limit".to_string(),
                "code": "CRF0002".to_string(),
                "status": StatusCode::PAYLOAD_TOO_LARGE.as_u16(),
            })),
            MiddlewareError::ParseFailed { error } => Json(json!({
                "error": self.to_string(),
                "cause": error.to_string(),
                "code": "CRF0003".to_string(),
                "status": StatusCode::BAD_REQUEST.as_u16(),
            })),
            MiddlewareError::ValidationFailed { error } => Json(json!({
                "error": self.to_string(),
                "cause(s)": error,
                "code": "CRF0004".to_string(),
                "status": StatusCode::BAD_REQUEST.as_u16(),
            })),
        }
    }
}

impl IntoResponse for MiddlewareError {
    fn into_response(self) -> Response {
        let body = self.get_response_json();
        let status = body.0.get("status").and_then(|s| s.as_u64()).unwrap_or(400) as u16;
        (
            StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_REQUEST),
            body,
        )
            .into_response()
    }
}
