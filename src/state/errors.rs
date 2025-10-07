use thiserror::Error;

#[derive(Debug, Error)]
pub enum StateError {
    #[error("No repo found for the given state {0}")]
    StateNotConfigured(String),
    #[error("Error getting repo for path {path:?}. Root Error {root_error:?}")]
    RepoError { path: String, root_error: String },
    #[error("Error getting store for path {path:?}. Root Error {root_error:?}")]
    StoreError { path: String, root_error: String },
    #[error("Error getting schema {0}")]
    SchemaServiceError(String),
    #[error("Invalid Schema. object_schema for service not found")]
    InvalidSchema,
    #[error("Schema could not be compiled {0}")]
    SchemaCompilationError(String),
}
