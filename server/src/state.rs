use std::time::Duration;

use crate::shared::config::Config;
use crate::shared::ws::hub::Hub;
use sqlx::PgPool;

const HTTP_CLIENT_TIMEOUT_SECS: u64 = 10;

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
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(HTTP_CLIENT_TIMEOUT_SECS))
                .build()
                .expect("failed to build the shared HTTP client"),
        }
    }
}
