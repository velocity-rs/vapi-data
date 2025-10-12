use crate::db::errors::MongoError;
use crate::db::{Repository, RepositoryError};
use crate::utils::bson;
impl Repository {
    pub async fn create(
        &self,
        value: &serde_json::Value,
    ) -> Result<serde_json::Value, RepositoryError> {
        let doc = bson::to_doc(value)
            .map_err(|e| RepositoryError::BsonSerializationFailed(e.to_string()))?;

        self.collection
            .insert_one(doc)
            .await
            .map_err(|e| RepositoryError::DatabaseError(MongoError::from(e)))?;
        Ok(value.clone())
    }
}
