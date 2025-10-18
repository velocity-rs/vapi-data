use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use mongodb::error::{Error, ErrorKind, WriteError, WriteFailure};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateError {
    #[error("Could not parse request body {0}")]
    BodyParserFailed(String),

    #[error("Duplicate Record: {0}")]
    DuplicateRecord(String),

    #[error("Transaction Error: {0}")]
    TransactionError(String),

    #[error("Write Concern Constraint Failed: {0}. You may want to retry the operation.")]
    WriteConcernError(String),

    #[error("Unexpected Server Error: {0}")]
    UnexpectedError(String),
}

impl CreateError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            CreateError::BodyParserFailed(_) => StatusCode::BAD_REQUEST,
            CreateError::DuplicateRecord(_) => StatusCode::BAD_REQUEST,
            CreateError::TransactionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CreateError::WriteConcernError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CreateError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<Error> for CreateError {
    fn from(e: Error) -> Self {
        let kind = *e.clone().kind;
        match kind {
            ErrorKind::Write(write_error) => match write_error {
                WriteFailure::WriteError(WriteError { code, message, .. }) => {
                    if code == 11000 {
                        CreateError::DuplicateRecord(message)
                    } else {
                        CreateError::UnexpectedError(message)
                    }
                }
                WriteFailure::WriteConcernError(write_concern_error) => {
                    CreateError::WriteConcernError(write_concern_error.message)
                }
                _ => CreateError::UnexpectedError("Unexpected write failure".to_string()),
            },
            ErrorKind::Transaction { message, .. } => {
                CreateError::TransactionError(message.clone())
            }

            _ => CreateError::UnexpectedError(format!("MongoDB Error: {}", e)),
        }
    }
}

impl IntoResponse for CreateError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = axum::Json(serde_json::json!({ "error": self.to_string() }));
        (status, body).into_response()
    }
}
