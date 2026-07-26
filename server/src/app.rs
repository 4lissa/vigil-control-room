use crate::state::AppState;
use axum::{Router, routing::get};

pub fn build_router(_state: AppState) -> Router {
    Router::new().route("/health", get(health))
}

async fn health() -> &'static str {
    "ok"
}
