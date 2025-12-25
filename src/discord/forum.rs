use twilight_model::channel::message::embed::{Embed, EmbedFooter};
use twilight_model::id::{marker::ChannelMarker, Id};

use crate::discord::client::DiscordClient;
use crate::error::{Error, Result};

/// Colors for CI status
pub const COLOR_SUCCESS: u32 = 0x2ECC71; // Green
pub const COLOR_FAILURE: u32 = 0xE74C3C; // Red
pub const COLOR_SKIPPED: u32 = 0x95A5A6; // Grey
pub const COLOR_PR: u32 = 0x9B59B6; // Purple - PR merged
pub const COLOR_BOUNTY: u32 = 0xF1C40F; // Gold - Bounty events
pub const COLOR_ISSUE: u32 = 0x3498DB; // Blue - Other issues

impl DiscordClient {
    pub async fn create_forum_thread(
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

    /// Create a forum thread with a styled embed (colored bar)
    pub async fn create_forum_thread_with_embed(
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

    pub async fn send_message(&self, channel_id: Id<ChannelMarker>, content: &str) -> Result<()> {
        self.http
            .create_message(channel_id)
            .content(content)
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;
        Ok(())
    }

    /// Send a styled embed message to an existing thread
    pub async fn send_message_with_embed(
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

    /// Secure a thread (Lock + Keep Unarchived + Manual Pin)
    pub async fn secure_thread(&self, thread_id: Id<ChannelMarker>) -> Result<()> {
        // 1. Standard library call for Locked + Unarchived
        self.http
            .update_thread(thread_id)
            .archived(false)
            .locked(true)
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;

        // 2. Manual HTTP Patch for Pinning (Flag: 1 << 1)
        // This is a workaround for library version limitations.
        let url = format!("https://discord.com/api/v10/channels/{}", thread_id);
        let _ = reqwest::Client::new()
            .patch(&url)
            .header("Authorization", format!("Bot {}", self.token))
            .json(&serde_json::json!({ "flags": 2 }))
            .send()
            .await;

        Ok(())
    }
}

pub fn format_release(event: &crate::github::events::ReleaseEvent) -> (String, String) {
    let title = format!("🚀 Release {}", event.release.tag_name);
    let body = format!(
        "**{}** released `{}`\n\n{}\n\n[View Release]({})",
        event.repository.full_name,
        event.release.tag_name,
        event.release.body.as_deref().unwrap_or(""),
        event.release.html_url
    );
    (title, body)
}

pub fn format_pr_merged(event: &crate::github::events::PullRequestEvent) -> (String, String) {
    let labels: Vec<&str> = event
        .pull_request
        .labels
        .iter()
        .map(|l| l.name.as_str())
        .collect();
    let label_str = if labels.is_empty() {
        String::new()
    } else {
        format!(" [{}]", labels.join(", "))
    };
    let title = format!("🧩 PR merged: #{}{}", event.pull_request.number, label_str);
    let body = format!(
        "**{}**\n\nMerged by @{}\n\n[View PR]({})",
        event.pull_request.title, event.sender.login, event.pull_request.html_url
    );
    (title, body)
}

pub fn format_issue(event: &crate::github::events::IssueEvent) -> (String, String) {
    let labels: Vec<&str> = event.issue.labels.iter().map(|l| l.name.as_str()).collect();
    let has_bounty = labels.iter().any(|l| *l == "bounty");
    let emoji = if has_bounty { "🪙" } else { "📋" };
    let title = format!(
        "{} Issue #{}: {}",
        emoji, event.issue.number, event.issue.title
    );
    let body = format!(
        "Opened by @{}\nLabels: {}\n\n[View Issue]({})",
        event.sender.login,
        labels.join(", "),
        event.issue.html_url
    );
    (title, body)
}

pub fn format_workflow(event: &crate::github::events::WorkflowRunEvent) -> (String, String) {
    let conclusion = event
        .workflow_run
        .conclusion
        .as_deref()
        .unwrap_or("unknown");
    let emoji = if conclusion == "success" {
        "✅"
    } else {
        "❌"
    };
    let title = format!(
        "{} CI: {}",
        emoji,
        event.workflow_run.name.as_deref().unwrap_or("Workflow")
    );
    let body = format!(
        "**{}** - {}\n\n[View Run]({})",
        event.repository.full_name, conclusion, event.workflow_run.html_url
    );
    (title, body)
}
