use std::sync::Arc;
use twilight_http::Client;
use twilight_model::channel::ChannelType;
use twilight_model::id::{
    marker::{ApplicationMarker, ChannelMarker, GuildMarker},
    Id,
};

use crate::error::{Error, Result};

#[derive(Clone)]
pub struct DiscordClient {
    pub http: Arc<Client>,
    pub application_id: Id<ApplicationMarker>,
}

impl DiscordClient {
    pub fn new(token: &str, application_id: u64) -> Self {
        let http = Arc::new(Client::new(token.to_string()));
        Self {
            http,
            application_id: Id::new(application_id),
        }
    }

    /// Create announcements channel (text channel)
    pub async fn create_announcements_channel(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<Id<ChannelMarker>> {
        let channel = self
            .http
            .create_guild_channel(guild_id, "announcements")
            .kind(ChannelType::GuildText)
            .await
            .map_err(|e| Error::Discord(e.to_string()))?
            .model()
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;

        Ok(channel.id)
    }

    /// Create GitHub forum channel
    pub async fn create_github_forum(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<Id<ChannelMarker>> {
        let channel = self
            .http
            .create_guild_channel(guild_id, "GitHub")
            .kind(ChannelType::GuildForum)
            .await
            .map_err(|e| Error::Discord(e.to_string()))?
            .model()
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;

        Ok(channel.id)
    }

    /// Create private Mod category with channels
    pub async fn create_mod_category(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<(Id<ChannelMarker>, Id<ChannelMarker>, Id<ChannelMarker>)> {
        // Create category
        let category = self
            .http
            .create_guild_channel(guild_id, "Mod")
            .kind(ChannelType::GuildCategory)
            .await
            .map_err(|e| Error::Discord(e.to_string()))?
            .model()
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;

        // Create project-review channel in category
        let review = self
            .http
            .create_guild_channel(guild_id, "project-review")
            .kind(ChannelType::GuildText)
            .parent_id(category.id)
            .await
            .map_err(|e| Error::Discord(e.to_string()))?
            .model()
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;

        // Create approvals channel in category
        let approvals = self
            .http
            .create_guild_channel(guild_id, "approvals")
            .kind(ChannelType::GuildText)
            .parent_id(category.id)
            .await
            .map_err(|e| Error::Discord(e.to_string()))?
            .model()
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;

        Ok((category.id, review.id, approvals.id))
    }

    /// Find channel by name in guild
    pub async fn find_channel_by_name(
        &self,
        guild_id: Id<GuildMarker>,
        name: &str,
    ) -> Result<Option<Id<ChannelMarker>>> {
        let channels = self
            .http
            .guild_channels(guild_id)
            .await
            .map_err(|e| Error::Discord(e.to_string()))?
            .model()
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;

        for channel in channels {
            if channel.name.as_deref() == Some(name) {
                return Ok(Some(channel.id));
            }
        }
        Ok(None)
    }
}
