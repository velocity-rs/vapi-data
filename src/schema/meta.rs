use std::sync::Arc;

use crate::schema::SchemaValidator;

use super::SchemaError;
use jsonschema::Validator;
use log::{debug, error, info, trace};
use serde_json::Value;

#[doc = "The MetasSchemas Struct holds the meta schemas"]
#[derive(Debug, Clone)]
pub(crate) struct MetaValidators {
    object: SchemaValidator,
    search: SchemaValidator,
    patch: SchemaValidator,
}

impl MetaValidators {
    pub(crate) async fn new() -> Result<Self, SchemaError> {
        const OBJECT_META_URL: &'static str = "object_meta_url";
        const SEARCH_META_URL: &'static str = "search_meta_url";
        const PATCH_META_URL: &'static str = "patch_meta_url";

        let object_schema_url = match config::get::<String>(OBJECT_META_URL) {
            Some(url) => url,
            None => {
                return Err(SchemaError::ConfigInvalid(OBJECT_META_URL.to_string()));
            }
        };

        debug!("Object Meta Schema url {}", object_schema_url);

        let object_meta = match Self::load_from_url(object_schema_url).await {
            Ok(value) => value,
            Err(e) => return Err(e),
        };

        trace!("Object Meta Schema Json {}", object_meta);
        info!("Compiling Object Meta Schema");

        let object_meta_validator = if let Ok(v) = Self::get_validator(&object_meta) {
            Arc::new(v)
        } else {
            error!("Error getting objcet validator");
            return Err(SchemaError::ObjectInvalid);
        };

        let search_meta_url = match config::get::<String>(SEARCH_META_URL) {
            Some(url) => url,
            None => {
                return Err(SchemaError::ConfigInvalid(SEARCH_META_URL.to_string()));
            }
        };

        debug!("Search Meta Schema url {}", search_meta_url);

        let search_meta = match Self::load_from_url(search_meta_url).await {
            Ok(value) => value,
            Err(e) => return Err(e),
        };

        trace!("Search Meta Schema Json {}", object_meta);
        info!("Compiling Search Meta Schema");

        let search_meta_validator = if let Ok(v) = Self::get_validator(&search_meta) {
            Arc::new(v)
        } else {
            error!("Error getting search validator");
            return Err(SchemaError::ObjectInvalid);
        };

        let patch_meta_url = match config::get::<String>(PATCH_META_URL) {
            Some(url) => url,
            None => {
                return Err(SchemaError::ConfigInvalid(PATCH_META_URL.to_string()));
            }
        };

        debug!("Patch Meta Schema url {}", patch_meta_url);

        let patch_meta = match Self::load_from_url(patch_meta_url).await {
            Ok(value) => value,
            Err(e) => return Err(e),
        };

        trace!("Patch Meta Schema Json {}", patch_meta);
        info!("Compiling Patch Meta Schema");

        let patch_meta_validator = if let Ok(v) = Self::get_validator(&patch_meta) {
            Arc::new(v)
        } else {
            return Err(SchemaError::ObjectInvalid);
        };

        let object_schema_validator = SchemaValidator {
            schema: object_meta,
            validator: object_meta_validator,
        };
        let search_schema_validator = SchemaValidator {
            schema: search_meta,
            validator: search_meta_validator,
        };

        let patch_schema_validator = SchemaValidator {
            schema: patch_meta,
            validator: patch_meta_validator,
        };

        return Ok(MetaValidators {
            object: object_schema_validator,
            search: search_schema_validator,
            patch: patch_schema_validator,
        });
    }

    /// This function loads the meta schemas after reading the url values from env
    /// If any of the keys are missing or any error occurs, one of the Schema
    async fn load_from_url(url: String) -> Result<Value, SchemaError> {
        match reqwest::get(url.clone()).await {
            Ok(response) => match response.json::<Value>().await {
                Ok(json_value) => {
                    return Ok(json_value);
                }
                Err(e) => {
                    error!("Error parsing url response {}", e);
                    return Err(SchemaError::ParsingFailed);
                }
            },
            Err(e) => return Err(SchemaError::ResponseFailed(e.to_string())),
        }
    }

    fn get_validator(schema: &Value) -> Result<Validator, SchemaError> {
        match jsonschema::draft202012::options()
            .with_draft(jsonschema::Draft::Draft202012)
            .should_validate_formats(true)
            .build(schema)
        {
            Ok(validator) => {
                log::info!("Schema compiled successfully {:?}", validator);
                Ok(validator)
            }
            Err(e) => {
                error!("Error compiling schema");
                Err(SchemaError::CompileFailed(e.to_string()))
            }
        }
    }

    pub fn validate_object(&self, instance: Value) -> bool {
        info!("Validating Object Schema againt Meta schema");
        self.object.validate(instance)
    }
}
