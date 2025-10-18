use std::fmt::Display;

use axum::http::StatusCode;
use axum_thiserror::ErrorStatus;
use log::trace;
use mongodb::error::{Error, ErrorKind, WriteError, WriteFailure};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::pagination::error::CursorError;

#[derive(Debug, Serialize, Deserialize)]
pub struct MongoError {
    kind: String,
    message: String,
}

impl Display for MongoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl From<mongodb::error::Error> for MongoError {
    fn from(e: mongodb::error::Error) -> Self {
        trace!("MongoDB Error: kind: {}, message: {}", e.kind, e);
        let kind = *e.kind;
        match kind {
            mongodb::error::ErrorKind::Write(write_error) => match write_error {
                WriteFailure::WriteError(WriteError { code, message, .. }) => {
                    if code == 11000 {
                        MongoError {
                            kind: "DuplicateKeyError".to_string(),
                            message,
                        }
                    } else {
                        MongoError {
                            kind: "WriteError".to_string(),
                            message,
                        }
                    }
                }
                WriteFailure::WriteConcernError(write_concern_error) => MongoError {
                    kind: "WriteConcernError".to_string(),
                    message: write_concern_error.message,
                },
                _ => MongoError {
                    kind: "WriteFailure".to_string(),
                    message: "Unexpected write failure".to_string(),
                },
            },
            _ => MongoError {
                kind: format!("{:?}", kind),
                message: "Unexpected error".to_string(),
            },
        }
    }
}

impl From<CursorError> for MongoError {
    fn from(e: CursorError) -> Self {
        return MongoError {
            kind: "CursorError".to_string(),
            message: e.to_string(),
        };
    }
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("A client error has occured {}: {}", .0.kind, .0.message)]
    ClientError(MongoError),

    #[error("A database error has occured {}: {}", .0.kind, .0.message)]
    DatabaseError(MongoError),
    #[error("DB repository is unusable because its not initialized. HINT: Use the init() function")]
    UnIntialized,
    #[error("Repository initialization failed: {}, {}", .0.kind, .0.message)]
    InitializationFailed(MongoError),
    #[error("Repository configuration error: {}", 0)]
    ConfigError(String),
    #[error("Error while running find command {}: {}", .0.kind, .0.message)]
    FindFailed(MongoError),
    #[error("Error during watch {}: {}", .0.kind, .0.message)]
    WatchError(MongoError),
    #[error("Error starting session {}: {}", .0.kind, .0.message)]
    SessionCreationError(MongoError),
    #[error("Error creating index {}: {}", .0.kind, .0.message)]
    IndexCreationError(MongoError),
    #[error("Repository is already initialized")]
    AlreadyInitialized,
    #[error("Cluster Healthcheck failed: {0}")]
    PingFailed(String),
}
