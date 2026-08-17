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
        .route("/teams/{team_id}/bans", get(handler::list_bans))
        .route(
            "/teams/{team_id}/members/{user_id}/kick",
            post(handler::kick_member),
        )
        .route(
            "/teams/{team_id}/members/{user_id}/ban",
            post(handler::ban_member).delete(handler::unban_member),
        )
}
