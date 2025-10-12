use crate::db::errors::MongoError;
use crate::db::{Repository, RepositoryError};
use crate::utils::bson;
use mongodb::bson::{bson, doc};
use mongodb::options::InsertOneOptions;
use serde_json::Value;
use uuid::Uuid;

impl Repository {
    pub async fn create(&self, value: &Value) -> Result<serde_json::Value, RepositoryError> {
        //add record_id created at and updated at fields
        let mut v = value.clone();
        let value = v.as_object_mut().unwrap();
        value.insert(
            "record_id".to_string(),
            Value::String(Uuid::new_v4().to_string()),
        );
        let now = chrono::Utc::now().to_rfc3339();
        value.insert("created_at".to_string(), Value::String(now.clone()));
        value.insert("updated_at".to_string(), Value::String(now));

        let doc = bson::to_doc(value)
            .map_err(|e| RepositoryError::BsonSerializationFailed(e.to_string()))?;

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
            Err(e) => Err(RepositoryError::DatabaseError(MongoError::from(e))),
        }
    }
}
