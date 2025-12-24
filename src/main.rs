mod config;
mod discord;
mod error;
mod github;
mod governance;
mod router;
mod storage;

use axum::{
    routing::{get, post},
    Json, Router,
};
use sqlx::PgPool;
use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::discord::client::DiscordClient;
use crate::discord::commands::handle_interaction;
use crate::github::webhook::handle_webhook;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub pool: PgPool,
    pub discord: DiscordClient,
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "bytehub",
        "version": VERSION
    }))
}

async fn root() -> &'static str {
    "⚡ ByteHub - GitHub → Governance → Discord"
}

use axum::extract::State;

async fn debug_env(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "discord_public_key_len": state.config.discord_public_key.len(),
        "discord_public_key_prefix": &state.config.discord_public_key[..8.min(state.config.discord_public_key.len())],
        "discord_application_id": state.config.discord_application_id,
        "discord_bot_token_len": state.config.discord_bot_token.len(),
        "database_url_set": !state.config.database_url.is_empty(),
        "github_webhook_secret_set": !state.config.github_webhook_secret.is_empty(),
    }))
}

fn print_banner(addr: &SocketAddr) {
    let display_host = if addr.ip().is_unspecified() {
        "localhost"
    } else {
        &addr.ip().to_string()
    };
    println!();
    println!("  \x1b[36m╔══════════════════════════════════════════╗\x1b[0m");
    println!("  \x1b[36m║\x1b[0m  \x1b[1;35m⚡ ByteHub\x1b[0m                              \x1b[36m║\x1b[0m");
    println!("  \x1b[36m║\x1b[0m  \x1b[90mGitHub → Governance → Discord\x1b[0m           \x1b[36m║\x1b[0m");
    println!("  \x1b[36m╚══════════════════════════════════════════╝\x1b[0m");
    println!();
    println!(
        "  \x1b[32m→\x1b[0m Server running at \x1b[1;4mhttp://{}:{}\x1b[0m",
        display_host,
        addr.port()
    );
    println!("  \x1b[32m→\x1b[0m Version: \x1b[33m{}\x1b[0m", VERSION);
    println!();
    println!("  \x1b[90mEndpoints:\x1b[0m");
    println!("    \x1b[32mGET \x1b[0m /                  \x1b[90m← Health check\x1b[0m");
    println!("    \x1b[32mGET \x1b[0m /health             \x1b[90m← JSON status\x1b[0m");
    println!("    \x1b[34mPOST\x1b[0m /webhooks/github   \x1b[90m← GitHub events\x1b[0m");
    println!("    \x1b[34mPOST\x1b[0m /webhooks/discord  \x1b[90m← Discord interactions\x1b[0m");
    println!();
    println!("  \x1b[90mPress Ctrl+C to stop\x1b[0m");
    println!();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = Config::from_env();
    let pool = storage::db::create_pool(&config.database_url).await?;
    let discord = DiscordClient::new(&config.discord_bot_token, config.discord_application_id);

    let state = AppState {
        config: config.clone(),
        pool,
        discord,
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/debug", get(debug_env))
        .route("/webhooks/github", post(handle_webhook))
        .route("/webhooks/discord", post(handle_interaction))
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;

    print_banner(&addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
