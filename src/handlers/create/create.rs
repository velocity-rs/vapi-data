use axum::{Json, extract::State, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::router::AppState;
use serde_json::{Value, json};

pub async fn create(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    // Validate the payload against the schema
    let is_valid = state.validator.is_valid(&payload);

    if !is_valid {
        let errors: Vec<String> = state
            .validator
            .iter_errors(&payload)
            .map(|e| e.to_string())
            .collect();
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({ "errors": errors })),
        );
    }

    // Insert the validated data into the database
    match state.repo.create(&payload).await {
        Ok(inserted_id) => (
            axum::http::StatusCode::CREATED,
            Json(json!({ "id": inserted_id })),
        ),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}
