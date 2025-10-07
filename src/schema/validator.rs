use serde::Serialize;
use serde_json::Value;

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

pub fn validate(schema: Value, instance: Value) -> Result<(), ValidationError> {
    // Get the validator for the schema
    //TODO this has to be optimized. Validator has to be cached in memory.
    let validator = match jsonschema::validator_for(&schema) {
        Ok(validator) => validator,
        Err(e) => return todo!(),
    };

    todo!()
}
