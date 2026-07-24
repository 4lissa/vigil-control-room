use axum::{Router, routing::get};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let app = Router::new().route("/health", get(health));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("failed to bind port 8080");

    info!("server listening on port 8080");

    axum::serve(listener, app)
        .await
        .expect("server error");
}

async fn health() -> &'static str {
    "ok"
}
