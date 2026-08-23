use axum::{
    Router,
    routing::{get, patch, post},
};

use crate::features::rule_engine::handler;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/teams/{team_id}/rules",
            get(handler::list_rules).post(handler::create_rule),
        )
        .route(
            "/teams/{team_id}/rules/{rule_id}",
            patch(handler::update_rule).delete(handler::delete_rule),
        )
}

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/webhooks/github/{rule_id}", post(handler::github_webhook))
        .route("/about.json", get(handler::about_json))
}
