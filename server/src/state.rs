use crate::shared::config::Config;
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: Config,
    pub db: PgPool,
}

impl AppState {
    pub fn new(config: Config, db: PgPool) -> Self {
        Self { config, db }
    }
}
