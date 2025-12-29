use convex::Value as ConvexValue;
use maplit::btreemap;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::storage::convex::ConvexDb;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub guild_id: String,
    pub announcements_id: String,
    pub github_forum_id: String,
}

/// Get server config by guild ID
pub async fn get_config(db: &ConvexDb, guild_id: &str) -> Result<Option<ServerConfig>> {
    let result = db
        .query(
            "serverConfig:get",
            btreemap! {
                "guild_id".into() => ConvexValue::String(guild_id.to_string()),
            },
        )
        .await?;

    if result.is_null() {
        return Ok(None);
    }

    let config: ServerConfig = serde_json::from_value(result)
        .map_err(|e| Error::InvalidPayload(format!("Failed to parse config: {}", e)))?;

    Ok(Some(config))
}

/// Save server config
pub async fn save_config(
    db: &ConvexDb,
    guild_id: &str,
    announcements_id: &str,
    github_forum_id: &str,
) -> Result<()> {
    db.mutation(
        "serverConfig:save",
        btreemap! {
            "guild_id".into() => ConvexValue::String(guild_id.to_string()),
            "announcements_id".into() => ConvexValue::String(announcements_id.to_string()),
            "github_forum_id".into() => ConvexValue::String(github_forum_id.to_string()),
        },
    )
    .await?;

    Ok(())
}
