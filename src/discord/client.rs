use crate::error::{Error, Result};
use async_trait::async_trait;
use std::sync::Arc;
use twilight_http::Client;
use twilight_model::channel::message::embed::{Embed, EmbedFooter};
use twilight_model::channel::ChannelType;
use twilight_model::id::{
    marker::{ApplicationMarker, ChannelMarker, GuildMarker},
    Id,
};

#[async_trait]
pub trait DiscordInterface: Send + Sync {
    async fn create_announcements_channel(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<Id<ChannelMarker>>;
    async fn create_github_category(&self, guild_id: Id<GuildMarker>) -> Result<Id<ChannelMarker>>;
    async fn create_project_forum(
        &self,
        guild_id: Id<GuildMarker>,
        category_id: Id<ChannelMarker>,
        project_name: &str,
    ) -> Result<Id<ChannelMarker>>;
    async fn create_mod_category(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<(Id<ChannelMarker>, Id<ChannelMarker>, Id<ChannelMarker>)>;
    async fn find_channel_by_name(
        &self,
        guild_id: Id<GuildMarker>,
        name: &str,
    ) -> Result<Option<Id<ChannelMarker>>>;
    async fn find_channel_containing(
        &self,
        guild_id: Id<GuildMarker>,
        keyword: &str,
    ) -> Result<Option<Id<ChannelMarker>>>;
    /// Find a category containing keyword (case-insensitive, only matches GuildCategory type)
    async fn find_category_containing(
        &self,
        guild_id: Id<GuildMarker>,
        keyword: &str,
    ) -> Result<Option<Id<ChannelMarker>>>;
    async fn create_channel_in_category(
        &self,
        guild_id: Id<GuildMarker>,
        category_id: Id<ChannelMarker>,
        name: &str,
    ) -> Result<Id<ChannelMarker>>;
    async fn find_active_thread_by_name(
        &self,
        guild_id: Id<GuildMarker>,
        parent_id: Id<ChannelMarker>,
        name: &str,
    ) -> Result<Option<Id<ChannelMarker>>>;
    async fn get_self_permissions(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<twilight_model::guild::Permissions>;
    async fn guild_channels(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<Vec<twilight_model::channel::Channel>>;
    fn application_id(&self) -> Id<ApplicationMarker>;

    // Forum & Messaging
    async fn create_forum_thread(
        &self,
        channel_id: Id<ChannelMarker>,
        name: &str,
        content: &str,
    ) -> Result<Id<ChannelMarker>>;
    async fn create_forum_thread_with_embed(
        &self,
        channel_id: Id<ChannelMarker>,
        thread_name: &str,
        title: &str,
        description: &str,
        color: u32,
        footer: Option<&str>,
    ) -> Result<Id<ChannelMarker>>;
    async fn send_message(&self, channel_id: Id<ChannelMarker>, content: &str) -> Result<()>;
    async fn send_message_with_embed(
        &self,
        thread_id: Id<ChannelMarker>,
        title: &str,
        description: &str,
        color: u32,
        footer: Option<&str>,
    ) -> Result<()>;
    async fn lock_thread(&self, thread_id: Id<ChannelMarker>) -> Result<()>;
    async fn pin_and_lock_thread(&self, thread_id: Id<ChannelMarker>) -> Result<()>;
}

#[derive(Clone)]
pub struct DiscordClient {
    pub http: Arc<Client>,
    pub application_id: Id<ApplicationMarker>,
    pub token: String,
}

impl DiscordClient {
    pub fn new(token: &str, application_id: u64) -> Self {
        let http = Arc::new(Client::new(token.to_string()));
        Self {
            http,
            application_id: Id::new(application_id),
            token: token.to_string(),
        }
    }
}

#[async_trait]
impl DiscordInterface for DiscordClient {
    /// Create announcements channel (text channel)
    async fn create_announcements_channel(
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

    /// Create GitHub category (container for project forums)
    async fn create_github_category(&self, guild_id: Id<GuildMarker>) -> Result<Id<ChannelMarker>> {
        let channel = self
            .http
            .create_guild_channel(guild_id, "GitHub")
            .kind(ChannelType::GuildCategory)
            .await
            .map_err(|e| Error::Discord(e.to_string()))?
            .model()
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;

        Ok(channel.id)
    }

    /// Create a forum channel for a project inside a category
    async fn create_project_forum(
        &self,
        guild_id: Id<GuildMarker>,
        category_id: Id<ChannelMarker>,
        project_name: &str,
    ) -> Result<Id<ChannelMarker>> {
        let channel = self
            .http
            .create_guild_channel(guild_id, project_name)
            .kind(ChannelType::GuildForum)
            .parent_id(category_id)
            .await
            .map_err(|e| Error::Discord(e.to_string()))?
            .model()
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;

        Ok(channel.id)
    }

    /// Create private Mod category with channels
    async fn create_mod_category(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<(Id<ChannelMarker>, Id<ChannelMarker>, Id<ChannelMarker>)> {
        use twilight_model::channel::permission_overwrite::{
            PermissionOverwrite, PermissionOverwriteType,
        };
        use twilight_model::guild::Permissions;

        let everyone_deny = PermissionOverwrite {
            id: guild_id.cast(),
            kind: PermissionOverwriteType::Role,
            allow: Permissions::empty(),
            deny: Permissions::VIEW_CHANNEL,
        };

        let category = self
            .http
            .create_guild_channel(guild_id, "Mod")
            .kind(ChannelType::GuildCategory)
            .permission_overwrites(&[everyone_deny])
            .await
            .map_err(|e| Error::Discord(e.to_string()))?
            .model()
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;

        // Create project-review channel in category (inherits permissions + explicit deny)
        let review = self
            .http
            .create_guild_channel(guild_id, "project-review")
            .kind(ChannelType::GuildText)
            .parent_id(category.id)
            .permission_overwrites(&[everyone_deny])
            .await
            .map_err(|e| Error::Discord(e.to_string()))?
            .model()
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;

        // Create approvals channel in category (inherits permissions + explicit deny)
        let approvals = self
            .http
            .create_guild_channel(guild_id, "approvals")
            .kind(ChannelType::GuildText)
            .parent_id(category.id)
            .permission_overwrites(&[everyone_deny])
            .await
            .map_err(|e| Error::Discord(e.to_string()))?
            .model()
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;

        Ok((category.id, review.id, approvals.id))
    }

    /// Find channel by name in guild (exact match)
    async fn find_channel_by_name(
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

    /// Find channel containing keyword (case-insensitive partial match)
    async fn find_channel_containing(
        &self,
        guild_id: Id<GuildMarker>,
        keyword: &str,
    ) -> Result<Option<Id<ChannelMarker>>> {
        let channels = self
            .http
            .guild_channels(guild_id)
            .await
            .map_err(|e| Error::Discord(e.to_string()))?
            .model()
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;

        let keyword_lower = keyword.to_lowercase();
        for channel in channels {
            if let Some(name) = channel.name.as_deref() {
                if name.to_lowercase().contains(&keyword_lower) {
                    return Ok(Some(channel.id));
                }
            }
        }
        Ok(None)
    }

    /// Find a category containing keyword (case-insensitive, only matches GuildCategory type)
    /// Use this when you specifically need a category, not any channel
    async fn find_category_containing(
        &self,
        guild_id: Id<GuildMarker>,
        keyword: &str,
    ) -> Result<Option<Id<ChannelMarker>>> {
        let channels = self
            .http
            .guild_channels(guild_id)
            .await
            .map_err(|e| Error::Discord(e.to_string()))?
            .model()
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;

        let keyword_lower = keyword.to_lowercase();
        for channel in channels {
            // Only match categories (not text channels, forums, etc.)
            if channel.kind != ChannelType::GuildCategory {
                continue;
            }
            if let Some(name) = channel.name.as_deref() {
                if name.to_lowercase().contains(&keyword_lower) {
                    return Ok(Some(channel.id));
                }
            }
        }
        Ok(None)
    }

    /// Create a text channel in an existing category
    async fn create_channel_in_category(
        &self,
        guild_id: Id<GuildMarker>,
        category_id: Id<ChannelMarker>,
        name: &str,
    ) -> Result<Id<ChannelMarker>> {
        let channel = self
            .http
            .create_guild_channel(guild_id, name)
            .kind(ChannelType::GuildText)
            .parent_id(category_id)
            .await
            .map_err(|e| Error::Discord(e.to_string()))?
            .model()
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;

        Ok(channel.id)
    }

    /// Find active thread by name in a channel (forum or text)
    async fn find_active_thread_by_name(
        &self,
        guild_id: Id<GuildMarker>,
        parent_id: Id<ChannelMarker>,
        name: &str,
    ) -> Result<Option<Id<ChannelMarker>>> {
        let threads = self
            .http
            .active_threads(guild_id)
            .await
            .map_err(|e| Error::Discord(e.to_string()))?
            .model()
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;

        for thread in threads.threads {
            if thread.parent_id == Some(parent_id) && thread.name.as_deref() == Some(name) {
                return Ok(Some(thread.id));
            }
        }
        Ok(None)
    }

    /// Get the bot's own permissions in a specific guild
    async fn get_self_permissions(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<twilight_model::guild::Permissions> {
        let member = self
            .http
            .guild_member(guild_id, self.application_id.cast())
            .await
            .map_err(|e| Error::Discord(e.to_string()))?
            .model()
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;

        // Calculate permissions based on roles and overrides
        // In a real scenario, we'd need more logic, but for simple bot roles:
        // Discord's guild_member endpoint doesn't return computed permissions.
        // However, we can use the roles to find the permissions.
        // Actually, the easiest way is to fetch the roles from the guild.
        let guild_roles = self
            .http
            .roles(guild_id)
            .await
            .map_err(|e| Error::Discord(e.to_string()))?
            .model()
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;

        let mut permissions = twilight_model::guild::Permissions::empty();
        for role_id in member.roles {
            if let Some(role) = guild_roles.iter().find(|r| r.id == role_id) {
                permissions |= role.permissions;
            }
        }
        // Also add @everyone permissions
        if let Some(everyone) = guild_roles.iter().find(|r| r.id == guild_id.cast()) {
            permissions |= everyone.permissions;
        }

        Ok(permissions)
    }

    async fn guild_channels(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<Vec<twilight_model::channel::Channel>> {
        let channels = self
            .http
            .guild_channels(guild_id)
            .await
            .map_err(|e| Error::Discord(e.to_string()))?
            .model()
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;
        Ok(channels)
    }

    fn application_id(&self) -> Id<ApplicationMarker> {
        self.application_id
    }

    async fn create_forum_thread(
        &self,
        channel_id: Id<ChannelMarker>,
        name: &str,
        content: &str,
    ) -> Result<Id<ChannelMarker>> {
        let thread = self
            .http
            .create_forum_thread(channel_id, name)
            .message()
            .content(content)
            .await
            .map_err(|e| Error::Discord(e.to_string()))?
            .model()
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;
        Ok(thread.channel.id)
    }

    async fn create_forum_thread_with_embed(
        &self,
        channel_id: Id<ChannelMarker>,
        thread_name: &str,
        title: &str,
        description: &str,
        color: u32,
        footer: Option<&str>,
    ) -> Result<Id<ChannelMarker>> {
        let embed = Embed {
            author: None,
            color: Some(color),
            description: Some(description.to_string()),
            fields: vec![],
            footer: footer.map(|f| EmbedFooter {
                icon_url: None,
                proxy_icon_url: None,
                text: f.to_string(),
            }),
            image: None,
            kind: "rich".to_string(),
            provider: None,
            thumbnail: None,
            timestamp: None,
            title: Some(title.to_string()),
            url: None,
            video: None,
        };

        let thread = self
            .http
            .create_forum_thread(channel_id, thread_name)
            .message()
            .embeds(&[embed])
            .await
            .map_err(|e| Error::Discord(e.to_string()))?
            .model()
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;
        Ok(thread.channel.id)
    }

    async fn send_message(&self, channel_id: Id<ChannelMarker>, content: &str) -> Result<()> {
        self.http
            .create_message(channel_id)
            .content(content)
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;
        Ok(())
    }

    async fn send_message_with_embed(
        &self,
        thread_id: Id<ChannelMarker>,
        title: &str,
        description: &str,
        color: u32,
        footer: Option<&str>,
    ) -> Result<()> {
        let embed = Embed {
            author: None,
            color: Some(color),
            description: Some(description.to_string()),
            fields: vec![],
            footer: footer.map(|f| EmbedFooter {
                icon_url: None,
                proxy_icon_url: None,
                text: f.to_string(),
            }),
            image: None,
            kind: "rich".to_string(),
            provider: None,
            thumbnail: None,
            timestamp: None,
            title: Some(title.to_string()),
            url: None,
            video: None,
        };

        self.http
            .create_message(thread_id)
            .embeds(&[embed])
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;
        Ok(())
    }

    /// Lock thread only (for sidebar threads - PRs, Issues, CI status, etc.)
    async fn lock_thread(&self, thread_id: Id<ChannelMarker>) -> Result<()> {
        // Lock the thread so only moderators can post
        self.http
            .update_thread(thread_id)
            .archived(false)
            .locked(true)
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;

        Ok(())
    }

    /// Pin and lock thread (for Activity thread only - Discord allows only 1 pinned thread per forum)
    async fn pin_and_lock_thread(&self, thread_id: Id<ChannelMarker>) -> Result<()> {
        use twilight_model::channel::ChannelFlags;

        // Lock the thread
        self.http
            .update_thread(thread_id)
            .archived(false)
            .locked(true)
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;

        // Pin the thread (Discord allows only 1 pinned thread per forum)
        self.http
            .update_channel(thread_id)
            .flags(ChannelFlags::PINNED)
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;

        Ok(())
    }
}
