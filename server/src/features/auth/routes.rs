use axum::{
    Router,
    routing::{get, patch, post},
};

use crate::AppState;
use crate::features::auth::handler;

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/register", post(handler::register))
        .route("/login", post(handler::login))
}

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/logout", post(handler::logout))
        .route("/me", get(handler::me))
        .route("/me", patch(handler::update_me))
}
