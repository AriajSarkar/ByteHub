use async_trait::async_trait;
use bytehub::error::Result;
use twilight_model::guild::Permissions;
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker},
    Id,
};

use bytehub::discord::client::DiscordInterface;
use twilight_model::id::marker::ApplicationMarker;

pub struct MockDiscord {
    pub permissions: Permissions,
    pub fail_all: bool,
}

/// Initialize environment variables from .env file
fn init_env() {
    dotenvy::dotenv().ok();
}

/// Create a test AppState with MockDiscord (handles env init internally)
pub fn create_state(pool: sqlx::PgPool, permissions: Permissions) -> bytehub::AppState {
    init_env();
    let discord = std::sync::Arc::new(MockDiscord {
        permissions,
        fail_all: false,
    });
    bytehub::AppState {
        config: bytehub::config::Config::from_env(),
        pool,
        discord,
    }
}

#[async_trait]
impl DiscordInterface for MockDiscord {
    async fn get_self_permissions(&self, _guild_id: Id<GuildMarker>) -> Result<Permissions> {
        if self.fail_all {
            return Err(bytehub::error::Error::Discord("Mock failure".into()));
        }
        Ok(self.permissions)
    }
    async fn create_announcements_channel(
        &self,
        _guild_id: Id<GuildMarker>,
    ) -> Result<Id<ChannelMarker>> {
        Ok(Id::new(100))
    }
    async fn create_github_category(
        &self,
        _guild_id: Id<GuildMarker>,
    ) -> Result<Id<ChannelMarker>> {
        Ok(Id::new(200))
    }
    async fn create_project_forum(
        &self,
        _guild_id: Id<GuildMarker>,
        _c: Id<ChannelMarker>,
        _n: &str,
    ) -> Result<Id<ChannelMarker>> {
        Ok(Id::new(300))
    }
    async fn create_mod_category(
        &self,
        _guild_id: Id<GuildMarker>,
    ) -> Result<(Id<ChannelMarker>, Id<ChannelMarker>, Id<ChannelMarker>)> {
        Ok((Id::new(400), Id::new(401), Id::new(402)))
    }
    async fn find_channel_by_name(
        &self,
        _guild_id: Id<GuildMarker>,
        _name: &str,
    ) -> Result<Option<Id<ChannelMarker>>> {
        Ok(None)
    }
    async fn create_channel_in_category(
        &self,
        _guild_id: Id<GuildMarker>,
        _category_id: Id<ChannelMarker>,
        _name: &str,
    ) -> Result<Id<ChannelMarker>> {
        Ok(Id::new(500))
    }
    async fn find_active_thread_by_name(
        &self,
        _guild_id: Id<GuildMarker>,
        _parent_id: Id<ChannelMarker>,
        _name: &str,
    ) -> Result<Option<Id<ChannelMarker>>> {
        Ok(None)
    }
    async fn guild_channels(
        &self,
        _guild_id: Id<GuildMarker>,
    ) -> Result<Vec<twilight_model::channel::Channel>> {
        Ok(vec![])
    }
    fn application_id(&self) -> Id<ApplicationMarker> {
        Id::new(123)
    }

    // Forum & Messaging
    async fn create_forum_thread(
        &self,
        _channel_id: Id<ChannelMarker>,
        _name: &str,
        _content: &str,
    ) -> Result<Id<ChannelMarker>> {
        Ok(Id::new(600))
    }
    async fn create_forum_thread_with_embed(
        &self,
        _channel_id: Id<ChannelMarker>,
        _thread_name: &str,
        _title: &str,
        _description: &str,
        _color: u32,
        _footer: Option<&str>,
    ) -> Result<Id<ChannelMarker>> {
        Ok(Id::new(700))
    }
    async fn send_message(&self, _channel_id: Id<ChannelMarker>, _content: &str) -> Result<()> {
        Ok(())
    }
    async fn send_message_with_embed(
        &self,
        _thread_id: Id<ChannelMarker>,
        _title: &str,
        _description: &str,
        _color: u32,
        _footer: Option<&str>,
    ) -> Result<()> {
        Ok(())
    }
    async fn secure_thread(&self, _thread_id: Id<ChannelMarker>) -> Result<()> {
        Ok(())
    }
}
