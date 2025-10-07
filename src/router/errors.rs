use axum::http::StatusCode;
use axum_thiserror::ErrorStatus;
use thiserror::Error;

#[derive(Debug, Error, ErrorStatus)]
pub enum StateError {
    #[error("Invalid Path {0}")]
    #[status(StatusCode::BAD_REQUEST)]
    InvalidPath(String),
    #[error("No repo found for the given state {0}")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    StateNotConfigured(String),
    #[error("State Error")]
    #[status(StatusCode::BAD_REQUEST)]
    StateError,
}
