pub mod config;
pub mod discord;
pub mod error;
pub mod github;
pub mod governance;
pub mod router;
pub mod storage;

use crate::config::Config;
use crate::discord::client::DiscordInterface;
use crate::discord::commands::handle_interaction;
use crate::github::webhook::handle_webhook;
use axum::{
    routing::{get, post},
    Json, Router,
};
use sqlx::PgPool;
use std::sync::Arc;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub pool: PgPool,
    pub discord: Arc<dyn DiscordInterface>,
}

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "bytehub",
        "version": VERSION
    }))
}

pub async fn root() -> &'static str {
    "⚡ ByteHub - GitHub → Governance → Discord"
}

use axum::extract::State;

pub async fn debug_env(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "discord_public_key_len": state.config.discord_public_key.len(),
        "discord_public_key_prefix": &state.config.discord_public_key[..8.min(state.config.discord_public_key.len())],
        "discord_application_id": state.config.discord_application_id,
        "discord_bot_token_len": state.config.discord_bot_token.len(),
        "database_url_set": !state.config.database_url.is_empty(),
        "github_webhook_secret_set": !state.config.github_webhook_secret.is_empty(),
    }))
}

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/debug", get(debug_env))
        .route("/webhooks/github", post(handle_webhook))
        .route("/webhooks/discord", post(handle_interaction))
        .with_state(state)
}
