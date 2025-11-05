use std::fmt::Display;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StateError {
    #[error("Schema Load Failed")]
    SchemaLoadFailed,
    #[error("Schema Compile Failed")]
    SchemaCompileFailed,
    #[error("Repository Initialization Failed")]
    RepoInitFailed,
    #[error("Schema 'object' not found or invalid")]
    InvalidObjectSchema,
}

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Could not bind address")]
    AddrBindingFailed(String),
    #[error("Could not retreive local address")]
    LocalAddressFailure(String),
    #[error("Path Config Error. {0} not configured")]
    PathConfigError(String),
}
