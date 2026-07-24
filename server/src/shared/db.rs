use sqlx::PgPool;

pub async fn create_pool(database_url: &str) -> PgPool {
    PgPool::connect(database_url)
        .await
        .expect("failed to connect to database")
}
