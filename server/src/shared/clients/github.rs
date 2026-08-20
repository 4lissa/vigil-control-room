use serde::Deserialize;
use url::Url;
use uuid::Uuid;

use crate::shared::error::AppError;

const USER_AGENT: &str = "vigil";

pub struct UserProfile {
    pub id: u64,
    pub login: String,
    pub email: Option<String>,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct RawUserProfile {
    id: u64,
    login: String,
    email: Option<String>,
}

#[derive(Deserialize)]
struct Email {
    email: String,
    primary: bool,
    verified: bool,
}

pub fn authorize_url(client_id: &str, redirect_uri: &str) -> (String, String) {
    let state = Uuid::new_v4().to_string();

    let mut url = Url::parse("https://github.com/login/oauth/authorize")
        .expect("hardcoded GitHub authorize URL must be valid");
    url.query_pairs_mut()
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("scope", "read:user user:email")
        .append_pair("state", &state);

    (url.to_string(), state)
}

pub async fn exchange_code_for_token(
    http_client: &reqwest::Client,
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    code: &str,
) -> Result<String, AppError> {
    let response = http_client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("code", code),
            ("redirect_uri", redirect_uri),
        ])
        .send()
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, "failed to reach GitHub token endpoint");
            AppError::InternalError
        })?;

    if !response.status().is_success() {
        tracing::error!(status = %response.status(), "GitHub token endpoint returned an error status");
        return Err(AppError::InternalError);
    }

    let token: TokenResponse = response.json().await.map_err(|e| {
        tracing::error!(error = ?e, "failed to parse GitHub token response, code may be invalid");
        AppError::Unauthorized
    })?;

    Ok(token.access_token)
}

pub async fn fetch_profile(
    http_client: &reqwest::Client,
    access_token: &str,
) -> Result<UserProfile, AppError> {
    let mut profile: RawUserProfile = http_client
        .get("https://api.github.com/user")
        .bearer_auth(access_token)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, "failed to reach GitHub user endpoint");
            AppError::InternalError
        })?
        .json()
        .await
        .map_err(|e| {
            tracing::error!(error = ?e, "failed to parse GitHub user response");
            AppError::InternalError
        })?;

    if profile.email.is_none() {
        profile.email = fetch_primary_email(http_client, access_token).await;
    }

    Ok(UserProfile {
        id: profile.id,
        login: profile.login,
        email: profile.email,
    })
}

async fn fetch_primary_email(http_client: &reqwest::Client, access_token: &str) -> Option<String> {
    let emails: Vec<Email> = http_client
        .get("https://api.github.com/user/emails")
        .bearer_auth(access_token)
        .header("User-Agent", USER_AGENT)
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;

    emails
        .into_iter()
        .find(|e| e.primary && e.verified)
        .map(|e| e.email)
}
