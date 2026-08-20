use axum::http::HeaderValue;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub cors_origin: HeaderValue,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub github_redirect_uri: String,
    pub token_encryption_key: [u8; 32],
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            database_url: required_env("DATABASE_URL"),
            server_port: optional_env("SERVER_PORT", "8080")
                .parse()
                .expect("SERVER_PORT must be a valid port number"),
            cors_origin: HeaderValue::from_str(&optional_env(
                "CORS_ORIGIN",
                "http://localhost:3000",
            ))
            .expect("CORS_ORIGIN must be a valid header value"),
            github_client_id: required_env("GITHUB_CLIENT_ID"),
            github_client_secret: required_env("GITHUB_CLIENT_SECRET"),
            github_redirect_uri: optional_env(
                "GITHUB_REDIRECT_URI",
                "http://localhost:8080/github/callback",
            ),
            token_encryption_key: derive_key(&required_env("TOKEN_ENCRYPTION_KEY")),
        }
    }
}

fn derive_key(secret: &str) -> [u8; 32] {
    Sha256::digest(secret.as_bytes()).into()
}

fn required_env(key: &str) -> String {
    let value = std::env::var(key).unwrap_or_else(|_| panic!("{key} must be set"));
    if value.is_empty() {
        panic!("{key} must not be empty");
    }
    value
}

fn optional_env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
