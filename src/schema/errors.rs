use serde::Serialize;
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

#[derive(Serialize)]
pub struct ValidationError {
    pub req_id: String,
    pub errors: Vec<Error>,
}

#[derive(Serialize)]
pub struct Error {
    pub message: String,
    pub location: String,
}

impl ValidationError {
    pub fn new(req_id: String, error_msgs: Vec<(String, String)>) -> Self {
        let errors = error_msgs
            .iter()
            .map(|e| Error {
                message: e.to_owned().0,
                location: e.to_owned().1,
            })
            .collect::<Vec<Error>>();

        Self { req_id, errors }
    }

    pub fn is_null(&self) -> bool {
        self.errors.is_empty()
    }
}
