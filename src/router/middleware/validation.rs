use std::usize;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::IntoResponse,
};
use log::debug;

use crate::{
    router::{AppState, middleware::errors::MiddlewareError},
    schema,
};

use axum::body::to_bytes;
use serde_json::Value;

pub trait Validator {
    fn new(schema: Value) -> Self;
    fn is_valid(&self, value: &Value) -> bool;
    fn iter_errors(&self, value: &Value) -> Vec<String>;
}

pub async fn validate_req_body(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> impl IntoResponse {
    debug!("Validating request body");
    let (parts, body) = req.into_parts();
    //TODO: Match method and process differently for GET/DELETE vs POST/PUT/PATCH

    let bytes = to_bytes(body, usize::MAX).await;
    match bytes {
        Ok(bytes) => {
            let json_result: Result<Value, _> = serde_json::from_slice(&bytes);
            match json_result {
                Ok(ref value) => {
                    if state.object_validator.is_valid(value) {
                        debug!("Request body is valid");
                        // Reconstruct the request with the original body for downstream handlers
                        let new_body = axum::body::Body::from(bytes);
                        req = Request::from_parts(parts, new_body);
                        return next.run(req).await;
                    } else {
                        let errors: Vec<String> = state
                            .object_validator
                            .iter_errors(value)
                            .map(|e| e.to_string())
                            .map(|s| s.replace("\"", ""))
                            .collect();
                        debug!("Request body validation failed: {:?}", errors);
                        return MiddlewareError::ValidationFailed { error: errors }.into_response();
                    }
                }
                Err(e) => {
                    debug!("Failed to parse JSON body for validation");
                    return MiddlewareError::ParseFailed { error: e }.into_response();
                }
            }
        }
        Err(e) => MiddlewareError::ReadFailed { error: e }.into_response(),
    }
}
