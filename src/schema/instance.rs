use std::sync::Arc;

use log::{error, info};
use serde_json::Value;
use tokio::{fs::File, io::AsyncReadExt};

use super::SchemaValidator;

use super::SchemaError;
#[derive(Debug)]
pub struct InstanceValidators {
    object: SchemaValidator,
    search: Option<SchemaValidator>,
    patch: Option<SchemaValidator>,
}

impl InstanceValidators {
    pub async fn new() -> Result<Self, SchemaError> {
        const INSTANCE_SCHEMA_FILE: &str = "instance_schema_file";
        let instance_schema_file = match config::get::<String>(INSTANCE_SCHEMA_FILE) {
            Some(url) => url,
            None => {
                return Err(SchemaError::ConfigInvalid(INSTANCE_SCHEMA_FILE.to_string()));
            }
        };

        let object_instance_schema = match Self::read_to_json(instance_schema_file).await {
            Ok(value) => {
                info!("Instance read into json");
                value
            }
            Err(e) => {
                return Err(e);
            }
        };

        let object_instance_validator = match jsonschema::validator_for(&object_instance_schema) {
            Ok(validator) => Arc::new(validator),
            Err(e) => {
                error!("Error compiling Object Meta Schema for validation");
                return Err(SchemaError::CompileFailed(e.to_string()));
            }
        };

        let object_schema_validator = SchemaValidator {
            schema: object_instance_schema,
            validator: object_instance_validator,
        };

        return Ok(InstanceValidators {
            object: object_schema_validator,
            search: Option::None,
            patch: Option::None,
        });
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

    pub fn get_object_instance_schema(&self) -> Value {
        self.object.schema.clone()
    }
}
