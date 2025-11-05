use std::hash::{Hash, Hasher};

use crate::router::AppState;
use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use etag::EntityTag;
use log::debug;
use seahash::SeaHasher;
use serde_json::{Map, Value, json};
use uuid::Uuid;

pub async fn create(state: State<AppState>, body: Result<Json<Value>, JsonRejection>) -> Response {
    // Run pre-validation hooks here if any (not implemented yet)

    // Validate the payload against the schema

    let body = match body {
        Ok(json) => json.0,
        Err(e) => {
            debug!("Failed to validate payload: {:?}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };

    let is_valid = state.object_validator.is_valid(&body);

    if !is_valid {
        let errors: Vec<String> = state
            .object_validator
            .iter_errors(&body)
            .map(|e| e.to_string())
            .collect();
        return (StatusCode::BAD_REQUEST, Json(json!({ "errors": errors }))).into_response();
    }

    //add record_id created at and updated at fields
    let mut body_mut = body.clone();

    //generate etag from body

    let body_str = match serde_json::to_string(&body_mut) {
        Ok(s) => s,
        Err(e) => {
            debug!("Failed to serialize body for etag generation: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Failed to generate etag" })),
            )
                .into_response();
        }
    };

    let mut meta: Map<String, Value> = Map::new();

    meta.insert(
        "record_id".to_string(),
        Value::String(Uuid::new_v4().to_string()),
    );
    let now = chrono::Utc::now().to_rfc3339();
    meta.insert("created_at".to_string(), Value::String(now.clone()));
    meta.insert("updated_at".to_string(), Value::String(now));

    let mut hasher = SeaHasher::new();
    hasher.write(body_str.as_bytes());
    let hash = format!("{:x}", hasher.finish());
    let etag = EntityTag::weak(&hash);
    debug!("Generated etag: {}", etag.tag());

    meta.insert("etag".to_string(), Value::String(etag.tag().to_string()));

    if let Some(obj) = body_mut.as_object_mut() {
        obj.insert("meta".to_string(), Value::Object(meta));
    }

    match state.repo.create(&mut body_mut).await {
        Ok(result) => {
            return (StatusCode::CREATED, Json(result)).into_response();
        }
        Err(e) => return e.into_response(),
    }
}
