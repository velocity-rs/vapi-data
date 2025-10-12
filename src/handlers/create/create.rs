use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::router::AppState;
use serde_json::{Value, json};

pub async fn create(
    state: State<AppState>,
    body: Result<Json<Value>, JsonRejection>,
) -> impl IntoResponse {
    // Validate the payload against the schema

    let body = match body {
        Ok(json) => json.0,
        Err(e) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(json!({ "error": e.to_string() })),
            );
        }
    };

    let is_valid = state.validator.is_valid(&body);

    if !is_valid {
        let errors: Vec<String> = state
            .validator
            .iter_errors(&body)
            .map(|e| e.to_string())
            .collect();
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({ "errors": errors })),
        );
    }

    // Insert the validated data into the database
    match state.repo.create(&body).await {
        Ok(result) => (StatusCode::CREATED, Json(result)),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}
