use axum::{Router, middleware, routing::get};
use tower_http::cors::{CorsLayer, Any};
use axum::http::Method;

use crate::features::auth::routes as auth_routes;
use crate::shared::middleware::require_auth;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers(Any);

    let protected = Router::new()
        .merge(auth_routes::protected_router())
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));

    Router::new()
        .route("/health", get(health))
        .merge(auth_routes::public_router())
        .merge(protected)
        .layer(cors)
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}
