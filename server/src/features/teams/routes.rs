use axum::{
    Router,
    routing::{get, post},
};

use crate::AppState;
use crate::features::teams::handler;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/teams",
            post(handler::create_team).get(handler::get_my_teams),
        )
        .route("/teams/join", post(handler::join_team))
        .route("/teams/{team_id}", get(handler::get_team))
        .route("/teams/{team_id}/members", get(handler::get_members))
        .route(
            "/teams/{team_id}/invite",
            post(handler::generate_invitation_code),
        )
        .route("/teams/{team_id}/transfer", post(handler::transfer_manager))
}
