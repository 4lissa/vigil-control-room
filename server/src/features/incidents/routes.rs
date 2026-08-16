use axum::{
    Router,
    routing::{get, patch, post},
};

use crate::features::incidents::handler;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/teams/{team_id}/incidents",
            get(handler::list_incidents).post(handler::create_incident),
        )
        .route(
            "/teams/{team_id}/incidents/{incident_id}",
            get(handler::get_incident),
        )
        .route(
            "/teams/{team_id}/incidents/{incident_id}/acknowledge",
            post(handler::acknowledge_incident),
        )
        .route(
            "/teams/{team_id}/incidents/{incident_id}/escalate",
            post(handler::escalate_incident),
        )
        .route(
            "/teams/{team_id}/incidents/{incident_id}/resolve",
            post(handler::resolve_incident),
        )
        .route(
            "/teams/{team_id}/incidents/{incident_id}/assign",
            post(handler::assign_responder),
        )
        .route(
            "/teams/{team_id}/incidents/{incident_id}/timeline",
            get(handler::list_timeline_entries).post(handler::add_timeline_entry),
        )
        .route(
            "/teams/{team_id}/incidents/{incident_id}/timeline/{entry_id}",
            patch(handler::edit_timeline_entry),
        )
}
