use thiserror::Error;

#[derive(Debug, Error)]
pub enum StateError {
    #[error("Initialization Error {0}")]
    InitializationError(String),
    #[error("Invalid Path {0}")]
    InvalidPath(String),
    #[error("No repo found for the given state {0}")]
    StateNotConfigured(String),
    #[error("State Error")]
    StateError,
    #[error("Schema Load Failed")]
    SchemaLoadFailed,
    #[error("Schema Compile Failed")]
    SchemaCompileFailed,
    #[error("Repository Initialization Failed")]
    RepoInitFailed,
}
