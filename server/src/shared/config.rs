use axum::http::HeaderValue;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub cors_origin: HeaderValue,
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
        }
    }
}

fn required_env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("{key} must be set"))
}

fn optional_env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
