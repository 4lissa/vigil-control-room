use crate::shared::config::Config;
use crate::shared::ws::hub::Hub;
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: Config,
    pub db: PgPool,
    pub hub: Hub,
    pub http_client: reqwest::Client,
}

impl AppState {
    pub fn new(config: Config, db: PgPool) -> Self {
        Self {
            config,
            db,
            hub: Hub::new(),
            http_client: reqwest::Client::new(),
        }
    }
}
