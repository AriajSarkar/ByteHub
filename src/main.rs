use bytehub::config::Config;
use bytehub::discord::client::DiscordClient;
use bytehub::storage::convex::ConvexDb;
use bytehub::{create_app, AppState};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

const VERSION: &str = env!("CARGO_PKG_VERSION");

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
    println!("  \x1b[32m→\x1b[0m Database: \x1b[34mConvex\x1b[0m");
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
    let db = ConvexDb::new(&config.convex_url).await?;
    let discord = DiscordClient::new(&config.discord_bot_token, config.discord_application_id);

    let state = AppState {
        config: config.clone(),
        db,
        discord: Arc::new(discord),
    };

    let app = create_app(state);
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;

    print_banner(&addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
