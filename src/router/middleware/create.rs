use serde_json::Value;

use super::Validator;

pub struct ObjectValidator {
    schema: serde_json::Value,
}

impl Validator for ObjectValidator {
    fn new(schema: serde_json::Value) -> Self {
        Self { schema }
    }
    fn is_valid(&self, value: &Value) -> bool {
        // Implement your validation logic here
        true
    }

    fn iter_errors(&self, value: &Value) -> Vec<String> {
        // Implement your error extraction logic here
        vec![]
    }
}
