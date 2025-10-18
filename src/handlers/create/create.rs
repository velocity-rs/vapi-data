use crate::router::AppState;
use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use log::debug;
use serde_json::{Value, json};

pub async fn create(state: State<AppState>, body: Result<Json<Value>, JsonRejection>) -> Response {
    // Run pre-validation hooks here if any (not implemented yet)

    // Validate the payload against the schema
    debug!("Validating payload: {:?}", body);
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

    let is_valid = state.validator.is_valid(&body);

    if !is_valid {
        let errors: Vec<String> = state
            .validator
            .iter_errors(&body)
            .map(|e| e.to_string())
            .collect();
        return (StatusCode::BAD_REQUEST, Json(json!({ "errors": errors }))).into_response();
    }

    // Insert the validated data into the database
    match state.repo.create(&body).await {
        Ok(result) => {
            return (StatusCode::CREATED, Json(result)).into_response();
        }
        Err(e) => return e.into_response(),
    }
}
