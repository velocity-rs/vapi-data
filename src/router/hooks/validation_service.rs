use std::{
    pin::Pin,
    task::{Context, Poll},
    usize,
};

use axum::{
    Json,
    body::Body,
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use log::{debug, trace};
use serde_json::{Error, Value, json};
use tower::{Layer, Service};

use super::errors::HookError;

use crate::router::AppState;

#[derive(Clone)]
pub struct ValidationLayer {
    pub state: AppState,
}

impl<S> Layer<S> for ValidationLayer {
    type Service = ValidationMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        ValidationMiddleware {
            inner: service,
            state: self.state.clone(),
        }
    }
}

#[derive(Clone)]
pub struct ValidationMiddleware<S> {
    inner: S,
    state: AppState,
}

impl<S> Service<Request> for ValidationMiddleware<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut inner = self.inner.clone();
        let state = self.state.clone();
        Box::pin(async move {
            let (parts, body) = req.into_parts();
            //convert body to Value
            // get body into bytes
            let bytes = match axum::body::to_bytes(body, usize::MAX).await {
                Ok(bytes) => bytes,
                Err(err) => {
                    debug!("Failed to read request body: {}", err);
                    let response = HookError::ReadFailed(err.to_string()).into_response();
                    return Ok(response);
                }
            };

            // get Value from bytes
            let json: Value = if let Ok(body_json) = serde_json::from_slice(&bytes) {
                body_json
            } else {
                debug!("Failed to parse JSON body for validation");
                let response = HookError::ReadFailed("Failed to parse JSON".into()).into_response();
                return Ok(response);
            };

            /*if let Ok(json) = body_json {
                debug!("Validating payload: {:?}", json);
                let is_valid = state.validator.is_valid(&json);

                if !is_valid {
                    let errors: Vec<String> = state
                        .validator
                        .iter_errors(&json)
                        .map(|e| e.to_string())
                        .collect();
                    let response = (StatusCode::BAD_REQUEST, Json(json!({ "errors": errors })))
                        .into_response();
                    return Ok(response);
                }
            } else {
                debug!("Failed to parse JSON body for validation");
            }*/

            let req = Request::from_parts(parts, Body::from(bytes));
            return inner.call(req).await;
        })
    }
}
