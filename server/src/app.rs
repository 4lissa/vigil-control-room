use axum::{Router, routing::get};
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    Router::new().route("/health", get(health))
}

async fn health() -> &'static str {
    "ok"
}
