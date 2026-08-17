use axum::{Router, routing::get};

use crate::features::messages::handler;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/messages", get(handler::list_conversations))
        .route(
            "/messages/{user_id}",
            get(handler::get_conversation).post(handler::send_message),
        )
}
