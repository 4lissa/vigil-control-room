use axum::{
    Router,
    routing::{get, post},
};

use crate::features::releases::handler;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/teams/{team_id}/releases",
            get(handler::list_releases).post(handler::create_release),
        )
        .route(
            "/teams/{team_id}/releases/{release_id}",
            get(handler::get_release),
        )
        .route(
            "/teams/{team_id}/releases/{release_id}/steps",
            get(handler::list_release_steps),
        )
        .route(
            "/teams/{team_id}/releases/{release_id}/steps/{step_id}/validate",
            post(handler::validate_step),
        )
        .route(
            "/teams/{team_id}/releases/{release_id}/cancel",
            post(handler::cancel_release),
        )
}
