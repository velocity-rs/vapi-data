mod errors;
mod instance;
mod meta;
mod validator;

use std::sync::Arc;

use errors::SchemaError;
pub(crate) use instance::InstanceValidators;
use jsonschema::Validator;
pub(crate) use meta::MetaValidators;
use serde_json::Value;

#[derive(Debug, Clone)]
pub(crate) struct SchemaValidator {
    schema: Value,
    validator: Arc<Validator>,
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
