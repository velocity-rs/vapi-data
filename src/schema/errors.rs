use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("Schema URL Cofig is invalid `{0}`")]
    ConfigInvalid(String),
    #[error("Unable to get response from url")]
    ResponseFailed(String),
    #[error("Unable to parse json")]
    ParsingFailed,
    #[error("Unable to read file {0}")]
    ReadFailed(String),
    #[error("Error compiling validator {0}")]
    CompileFailed(String),
    #[error("Error compiling validator")]
    ObjectInvalid,
}
