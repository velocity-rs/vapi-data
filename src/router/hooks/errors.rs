use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum HookError {
    #[error("Failed to read request body: {0}")]
    ReadFailed(String),

    #[error("Body size exceeds limit")]
    BodySizeExceeded,
}

impl HookError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            HookError::ReadFailed(_) => StatusCode::BAD_REQUEST,
            HookError::BodySizeExceeded => StatusCode::BAD_REQUEST,
        }
    }
}

impl IntoResponse for HookError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = axum::Json(serde_json::json!({ "error": self.to_string() }));
        (status, body).into_response()
    }
}
