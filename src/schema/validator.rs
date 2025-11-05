use jsonschema::Validator as JsonValidator;
use serde_json::Value;
use std::sync::Arc;

use super::SchemaError;
pub struct Validator {
    schema: Value,
    validator: Arc<JsonValidator>,
}

impl Validator {
    pub async fn new(schema: Value) -> Result<Self, SchemaError> {
        let compiled =
            JsonValidator::new(&schema).map_err(|e| SchemaError::CompileFailed(e.to_string()))?;

        Ok(Validator {
            schema,
            validator: Arc::new(compiled),
        })
    }
}
