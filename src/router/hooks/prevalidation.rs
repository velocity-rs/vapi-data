use std::{
    pin::Pin,
    task::{Context, Poll},
};

use axum::{extract::Request, response::Response};
use log::{debug, trace};
use tower::{Layer, Service};

use crate::router::AppState;

#[derive(Clone)]
pub struct PreValidationLayer {
    pub state: AppState,
}

impl<S> Layer<S> for PreValidationLayer {
    type Service = PreValidationMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        PreValidationMiddleware {
            inner: service,
            state: self.state.clone(),
        }
    }
}

#[derive(Clone)]
pub struct PreValidationMiddleware<S> {
    inner: S,
    state: AppState,
}

impl<S> Service<Request> for PreValidationMiddleware<S>
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

    fn call(&mut self, req: Request) -> Self::Future {
        // Here you can add your pre-validation logic
        // For example, logging or modifying the request
        debug!("Running pre-validation hooks");
        let state = self.state.clone();

        let mut inner = self.inner.clone();
        Box::pin(async move { inner.call(req).await })
    }
}
