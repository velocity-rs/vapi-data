use std::sync::Arc;

use jsonschema::Validator;
use log::{debug, error, info, trace};
use serde_json::Value;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use super::errors::StateError;
use crate::config;
use crate::db::Repository;

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct AppState {
    pub validator: Arc<Validator>,
    pub schema: Value,
    pub repo: Arc<Repository>,
}

impl AppState {
    pub async fn new() -> Result<Self, StateError> {
        let schema_file = match config::get::<String>("SCHEMA_FILE") {
            Some(file) => file,
            None => {
                error!("SCHEMA_FILE not set in configuration");
                return Err(StateError::SchemaLoadFailed);
            }
        };

        trace!("Schema file path: {}", schema_file);

        let schema = match Self::read_to_json(schema_file).await {
            Ok(schema) => {
                debug!("Schema loaded");
                schema
            }
            Err(e) => {
                error!("Error loading schema: {}", e);
                return Err(StateError::SchemaLoadFailed);
            }
        };

        trace!("Schema content: {}", schema);

        let validator = match jsonschema::draft202012::options()
            .should_validate_formats(true)
            .build(&schema)
        {
            Ok(validator) => {
                debug!("Validator compiled");
                Arc::new(validator)
            }
            Err(e) => {
                error!("Error compiling validator: {}", e);
                return Err(StateError::SchemaCompileFailed);
            }
        };

        let repo = match Repository::init().await {
            Ok(repo) => {
                debug!("Repository initialized");
                Arc::new(repo)
            }
            Err(e) => {
                error!("Error initializing repository: {}", e);
                return Err(StateError::RepoInitFailed);
            }
        };

        trace!("Repository: {}", repo);

        Ok(AppState {
            validator,
            schema,
            repo,
        })
    }

    async fn read_to_json(file_name: String) -> Result<Value, StateError> {
        let mut file = File::open(file_name.clone()).await.map_err(|e| {
            error!("Error reading file {}", e);
            StateError::SchemaLoadFailed
        })?;
        let mut contents = Vec::new();
        let read_len = file.read_to_end(&mut contents).await.map_err(|e| {
            error!("Error reading file {}", e);
            StateError::SchemaLoadFailed
        })?;
        info!("Read {} bytes from file {}", read_len, file_name);
        let value = serde_json::from_slice::<Value>(&contents).map_err(|e| {
            error!("Error parsing JSON {}", e);
            StateError::SchemaLoadFailed
        })?;
        Ok(value)
    }
}
