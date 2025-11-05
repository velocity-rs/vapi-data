use std::sync::Arc;

use crate::config;
use jsonschema::Validator as JsonValidator;
use log::{error, info};
use serde_json::Value;
use tokio::{fs::File, io::AsyncReadExt};

use super::SchemaError;

const SCHEMA_ENV_VAR: &str = "SCHEMA_FILE";

#[derive(Debug)]
pub struct Validator {
    object: Vec<SchemaValidator>,
    search: Option<Vec<SchemaValidator>>,
    patch: Option<Vec<SchemaValidator>>,
}

#[derive(Debug, Clone)]
pub(crate) struct SchemaValidator {
    schema: Value,
    validator: Arc<JsonValidator>,
}

impl SchemaValidator {
    pub fn validate(&self, instance: Value) -> bool {
        match jsonschema::validate(&self.schema, &instance) {
            Ok(()) => {
                log::info!("Validation successful");
                true
            }
            Err(e) => {
                log::error!("Validation failed {}", e.to_string());
                false
            }
        }
    }
}

impl Validator {
    pub async fn new() -> Result<Self, SchemaError> {
        let schema_file = match config::get::<String>(SCHEMA_ENV_VAR) {
            Some(f) => f,
            None => {
                error!("Schema file not found");
                return Err(SchemaError::ConfigInvalid(SCHEMA_ENV_VAR.into()));
            }
        };

        let schemata = match Self::read_to_json(schema_file).await {
            Ok(value) => {
                info!("Schema read successfully");
                value
            }
            Err(e) => {
                return Err(e);
            }
        };

        if !schemata.is_array() {
            return Err(SchemaError::InvalidSchema {
                schema_name: "object".to_string(),
            });
        }

        object_validators = if let Some(schemata) = schemata.as_array() {
            schemata.iter().map(|s| {
                get_validator(s)
            }).collect();

        }

        let object_schema = if let Some(object_schema) = schemata.get("object") {
            object_schema
        } else {
            error!("Schema 'object' not found or invalid");
            return Err(SchemaError::InvalidObjectSchema);
        };

        let object_validator = match jsonschema::validator_for(&object_schema) {
            Ok(validator) => Arc::new(validator),
            Err(e) => {
                error!("Error compiling Object Meta Schema for validation");
                return Err(SchemaError::CompileFailed(e.to_string()));
            }
        };

        let object_schema_validator = SchemaValidator {};

        return Ok(Validator {
            object: object_schema_validator,
            search: None,
            patch: None,
        });
    }

    fn get_validator(s: &Value) -> Result<SchemaValidator, SchemaError> {

        let name = if let Some(n) = s.get("name"){
            if let Some(name) = n.as_str() {
                name.to_string()
            } else {
                return Err(SchemaError::InvalidObjectSchema)
            }
        } else {
            return Err(SchemaError::InvalidObjectSchema)
        };

        Err(SchemaError::InvalidObjectSchema)


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
