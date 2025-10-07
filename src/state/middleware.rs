use axum::{
    body::Body,
    extract::{Path, Request, State},
    middleware::Next,
    response::IntoResponse,
};
use log::trace;

use crate::state::AppState;
pub async fn state(
    State(app_state): State<AppState>,
    Path(path): Path<String>,
    req: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    trace!("Entering state middleware for path: {}", path);

    next.run(req).await
}
