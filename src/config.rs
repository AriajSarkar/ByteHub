use std::env;

#[derive(Clone)]
pub struct Config {
    pub convex_url: String,
    pub github_webhook_secret: String,
    pub discord_public_key: String,
    pub discord_bot_token: String,
    pub discord_application_id: u64,
    pub discord_invite: Option<String>,
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            convex_url: env::var("CONVEX_URL").expect("CONVEX_URL required"),
            github_webhook_secret: env::var("GITHUB_WEBHOOK_SECRET")
                .expect("GITHUB_WEBHOOK_SECRET required"),
            discord_public_key: env::var("DISCORD_PUBLIC_KEY")
                .expect("DISCORD_PUBLIC_KEY required"),
            discord_bot_token: env::var("DISCORD_BOT_TOKEN").expect("DISCORD_BOT_TOKEN required"),
            discord_application_id: env::var("DISCORD_APPLICATION_ID")
                .expect("DISCORD_APPLICATION_ID required")
                .parse()
                .expect("DISCORD_APPLICATION_ID must be a valid u64"),
            discord_invite: env::var("DISCORD_INVITE").ok(),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .unwrap_or(3000),
        }
    }
}
