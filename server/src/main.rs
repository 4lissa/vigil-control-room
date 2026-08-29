use tracing::info;
use tracing_subscriber::EnvFilter;
use vigil_server::AppState;
use vigil_server::shared::{config::Config, db::create_pool};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = Config::from_env();
    let db = create_pool(&config.database_url).await;

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("failed to run migrations");

    let port = config.server_port;
    let state = AppState::new(config, db);

    let app = vigil_server::build_router(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("failed to bind port");

    info!("server listening on port {port}");

    axum::serve(listener, app).await.expect("server error");
}
