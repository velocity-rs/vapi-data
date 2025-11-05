use log::error;
use serde_json::Value;

use crate::schema::errors::SchemaError;

use super::validator::Validator;
use crate::config;
use std::collections::HashMap;
use std::sync::Arc;

const OBJECT_SCHEMA_ENV_VAR: &str = "OBJECT_SCHEMA_FILE";
pub struct ObjectValidator {
    validators: HashMap<String, Arc<Validator>>,
}

impl ObjectValidator {
    pub async fn new() -> Result<Self, SchemaError> {
        let schema_file = match config::get::<String>(OBJECT_SCHEMA_ENV_VAR) {
            Some(f) => f,
            None => {
                error!("Error getting Object Schema File name from configuration");
                return Err(SchemaError::ConfigInvalid(format!(
                    "{} not found",
                    OBJECT_SCHEMA_ENV_VAR
                )));
            }
        };

        let schemata = match Self::read_to_json(schema_file).await {
            Ok(schemata) => schemata,
            Err(e) => {
                error!("Error reading schema file {}: {}", schema_file, e);
                return Err(SchemaError::ReadFailed(format!("{}: {}", schema_file, e)));
            }
        };

        todo!()
    }

    async fn read_to_json(file_name: String) -> Result<Value, SchemaError> {
        match File::open(file_name.clone()).await {
            Ok(mut file) => {
                let mut contents = Vec::new();
                match file.read_to_end(&mut contents).await {
                    Ok(read_len) => {
                        info!("Read {} bytes from file {}", read_len, file_name);
                        match serde_json::from_slice::<Value>(&contents) {
                            Ok(value) => return Ok(value),
                            Err(e) => {
                                error!("Error parsing JSON {}", e);
                                return Err(SchemaError::ParsingFailed);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error reading file {}", e);
                        return Err(SchemaError::ReadFailed(file_name));
                    }
                }
            }
            Err(e) => {
                error!("Error reading file {}", e);
                return Err(SchemaError::ReadFailed(file_name));
            }
        };
    }
}
