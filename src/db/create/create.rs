use super::errors::CreateError;
use crate::db::Repository;
use crate::utils::bson;
use mongodb::bson::bson;
use mongodb::options::InsertOneOptions;
use serde_json::Value;

impl Repository {
    pub async fn create(&self, value: &mut Value) -> Result<serde_json::Value, CreateError> {
        let value = value.as_object_mut().unwrap();

        let doc = bson::to_doc(value).map_err(|e| CreateError::BodyParserFailed(e.to_string()))?;

        let comment = Some(bson!({ "app": "vapi-data" }));

        let insert_options = InsertOneOptions::builder().comment(comment).build();

        match self
            .collection
            .insert_one(doc)
            .with_options(insert_options)
            .await
        {
            Ok(result) => {
                value.insert(
                    "_id".to_string(),
                    Value::String(result.inserted_id.to_string()),
                );
                Ok(value.clone().into())
            }
            Err(e) => Err(CreateError::from(e)),
        }
    }
}
