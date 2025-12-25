use sqlx::PgPool;
use tracing::info;
use twilight_model::id::Id;

use crate::discord::client::DiscordClient;
use crate::discord::forum::{COLOR_BOUNTY, COLOR_FAILURE, COLOR_ISSUE, COLOR_PR, COLOR_SUCCESS};
use crate::error::{Error, Result};
use crate::github::events::ParsedEvent;
use crate::governance::projects;

pub struct Dispatcher {
    pool: PgPool,
    discord: DiscordClient,
}

impl Dispatcher {
    pub fn new(pool: PgPool, discord: DiscordClient) -> Self {
        Self { pool, discord }
    }

    pub async fn dispatch(&self, event: ParsedEvent) -> Result<()> {
        let repo = match event.repo_full_name() {
            Some(r) => r.to_lowercase(),
            None => return Ok(()),
        };

        let project = match projects::get_approved_project(&self.pool, &repo).await? {
            Some(p) => p,
            None => {
                info!(repo, "event from unlisted/unapproved project, ignoring");
                return Ok(());
            }
        };

        // Ensure forum exists and is synced
        let guild_id = Id::new(project.guild_id.parse::<u64>().unwrap_or(0));
        let forum_id = self.ensure_forum_exists(&project, &repo, guild_id).await?;

        // 1. Log to the persistent "Project Activity" thread
        if !self.is_bot_actor(event.actor().unwrap_or("")) {
            let activity_tid_str = self
                .get_or_create_thread(&project, &repo, forum_id, guild_id)
                .await?;
            let activity_tid = Id::new(activity_tid_str.parse::<u64>().unwrap_or(0));
            if let Err(e) = self.post_event_to_thread(activity_tid, &event).await {
                info!(repo, error = %e, "failed to post to activity thread");
            } else {
                info!(repo, "logged event to project activity thread");
            }
        }

        // 2. Manage dedicated Sidebar threads for major milestones
        if self.should_post(&event) {
            if let Err(e) = self.manage_sidebar_thread(guild_id, forum_id, &event).await {
                info!(repo, error = %e, "failed to manage sidebar thread");
            } else {
                info!(repo, "handled sidebar thread for event");
            }
        }

        // 3. Post to announcements if applicable
        if self.should_announce(&event) {
            if let Err(e) = self.post_to_announcements(&event, &project).await {
                info!(repo, error = %e, "failed to post announcement");
            }
        }

        Ok(())
    }

    fn should_post(&self, event: &ParsedEvent) -> bool {
        match event {
            ParsedEvent::WorkflowRun(e) => {
                if e.action != "completed" {
                    return false;
                }
                let conclusion = e.workflow_run.conclusion.as_deref().unwrap_or("unknown");
                if conclusion == "skipped" || conclusion == "cancelled" {
                    return false;
                }
                let branch = e.workflow_run.head_branch.as_deref().unwrap_or("");
                branch == "main" || branch == "master"
            }
            ParsedEvent::PullRequest(e) => {
                if self.is_bot_actor(e.sender.login.as_str()) {
                    return false;
                }
                e.action == "opened"
                    || (e.action == "closed" && e.pull_request.merged.unwrap_or(false))
                    || e.action == "labeled"
            }
            ParsedEvent::Issue(e) => e.action == "opened" || e.action == "labeled",
            ParsedEvent::Release(e) => e.action == "published",
            ParsedEvent::Unknown => false,
        }
    }

    fn should_announce(&self, event: &ParsedEvent) -> bool {
        match event {
            ParsedEvent::Release(_) => true,
            ParsedEvent::Issue(e) => e.issue.labels.iter().any(|l| l.name == "bounty"),
            ParsedEvent::PullRequest(e) => e.pull_request.labels.iter().any(|l| l.name == "bounty"),
            _ => false,
        }
    }

    fn is_bot_actor(&self, login: &str) -> bool {
        let bots = ["dependabot", "renovate", "github-actions"];
        bots.iter().any(|b| login.to_lowercase().contains(b))
    }

    async fn ensure_forum_exists(
        &self,
        project: &projects::Project,
        repo: &str,
        guild_id: Id<twilight_model::id::marker::GuildMarker>,
    ) -> Result<Id<twilight_model::id::marker::ChannelMarker>> {
        let channels = self
            .discord
            .http
            .guild_channels(guild_id)
            .await
            .map_err(|e| Error::Discord(e.to_string()))?
            .model()
            .await
            .map_err(|e| Error::Discord(e.to_string()))?;

        if !project.forum_channel_id.is_empty() {
            if let Ok(id_u64) = project.forum_channel_id.parse::<u64>() {
                let id = Id::new(id_u64);
                if channels.iter().any(|c| c.id == id) {
                    return Ok(id);
                }
            }
        }

        // Forum missing, re-create
        info!(repo, "forum channel missing, re-creating and syncing");

        // Find or create category
        let category_id = match self
            .discord
            .find_channel_by_name(guild_id, "GitHub")
            .await?
        {
            Some(id) => id,
            None => self.discord.create_github_category(guild_id).await?,
        };

        let project_name = repo.split('/').last().unwrap_or(repo);
        let new_forum_id = self
            .discord
            .create_project_forum(guild_id, category_id, project_name)
            .await?;

        // Sync to DB
        projects::update_forum_id(&self.pool, repo, &new_forum_id.get().to_string()).await?;

        Ok(new_forum_id)
    }

    async fn get_or_create_thread(
        &self,
        project: &projects::Project,
        repo: &str,
        forum_id: Id<twilight_model::id::marker::ChannelMarker>,
        guild_id: Id<twilight_model::id::marker::GuildMarker>,
    ) -> Result<String> {
        let project_name = repo.split('/').last().unwrap_or(repo);
        let thread_name = format!("📦 {} Activity", project_name);

        // If thread ID exists in DB, verify it still exists in Discord
        if let Some(ref tid_str) = project.thread_id {
            if !tid_str.is_empty() {
                if let Ok(tid_u64) = tid_str.parse::<u64>() {
                    let tid = Id::new(tid_u64);

                    // We can't efficiently check if a specific ID is valid without a 404,
                    // so we use our find_active_thread_by_name helper.
                    if let Ok(Some(found_id)) = self
                        .discord
                        .find_active_thread_by_name(guild_id, forum_id, &thread_name)
                        .await
                    {
                        if found_id == tid {
                            return Ok(tid_str.clone());
                        }
                    }
                }
            }
        }

        // Create new thread if not found or stale
        let tid = self
            .discord
            .create_forum_thread(
                forum_id,
                &thread_name,
                "Project activity thread. All events will be posted here.",
            )
            .await?;

        let tid_str = tid.get().to_string();
        projects::update_thread_id(&self.pool, repo, &tid_str).await?;

        Ok(tid_str)
    }

    async fn post_event_to_thread(
        &self,
        thread_id: Id<twilight_model::id::marker::ChannelMarker>,
        event: &ParsedEvent,
    ) -> Result<()> {
        match event {
            ParsedEvent::WorkflowRun(e) => {
                let conclusion = e.workflow_run.conclusion.as_deref().unwrap_or("unknown");
                let color = if conclusion == "success" {
                    COLOR_SUCCESS
                } else {
                    COLOR_FAILURE
                };
                let emoji = if conclusion == "success" {
                    "✅"
                } else {
                    "❌"
                };
                let name = e.workflow_run.name.as_deref().unwrap_or("CI");
                let branch = e.workflow_run.head_branch.as_deref().unwrap_or("unknown");

                self.discord
                    .send_message_with_embed(
                        thread_id,
                        &format!("{} {} {}", emoji, name, conclusion),
                        &format!(
                            "Branch: `{}`\n[View Run]({})",
                            branch, e.workflow_run.html_url
                        ),
                        color,
                        None,
                    )
                    .await?;
            }
            ParsedEvent::PullRequest(e) => {
                let has_bounty = e.pull_request.labels.iter().any(|l| l.name == "bounty");
                let color = if has_bounty { COLOR_BOUNTY } else { COLOR_PR };
                let emoji = if has_bounty { "🪙" } else { "🧩" };
                let action_text = match e.action.as_str() {
                    "opened" => "opened",
                    "labeled" => "labeled",
                    "closed" if e.pull_request.merged.unwrap_or(false) => "merged",
                    "closed" => "closed",
                    _ => &e.action,
                };

                self.discord
                    .send_message_with_embed(
                        thread_id,
                        &format!("{} PR #{} {}", emoji, e.pull_request.number, action_text),
                        &format!(
                            "**{}**\nby @{}\n[View PR]({})",
                            e.pull_request.title, e.sender.login, e.pull_request.html_url
                        ),
                        color,
                        None,
                    )
                    .await?;
            }
            ParsedEvent::Issue(e) => {
                let has_bounty = e.issue.labels.iter().any(|l| l.name == "bounty");
                let color = if has_bounty {
                    COLOR_BOUNTY
                } else {
                    COLOR_ISSUE
                };
                let emoji = if has_bounty { "🪙" } else { "📋" };
                let action_text = match e.action.as_str() {
                    "opened" => "opened",
                    "labeled" => "labeled",
                    "closed" => "closed",
                    _ => &e.action,
                };

                self.discord
                    .send_message_with_embed(
                        thread_id,
                        &format!("{} Issue #{} {}", emoji, e.issue.number, action_text),
                        &format!(
                            "**{}**\nby @{}\n[View Issue]({})",
                            e.issue.title, e.sender.login, e.issue.html_url
                        ),
                        color,
                        None,
                    )
                    .await?;
            }
            ParsedEvent::Release(e) => {
                self.discord
                    .send_message_with_embed(
                        thread_id,
                        &format!("🚀 Release {}", e.release.tag_name),
                        &format!(
                            "{}\n\n[View Release]({})",
                            e.release.body.as_deref().unwrap_or(""),
                            e.release.html_url
                        ),
                        COLOR_SUCCESS,
                        Some(&format!("by @{}", e.sender.login)),
                    )
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn manage_sidebar_thread(
        &self,
        guild_id: Id<twilight_model::id::marker::GuildMarker>,
        forum_id: Id<twilight_model::id::marker::ChannelMarker>,
        event: &ParsedEvent,
    ) -> Result<()> {
        let (thread_name, title, description, color, footer_text) = match event {
            ParsedEvent::WorkflowRun(e) => {
                let conclusion = e.workflow_run.conclusion.as_deref().unwrap_or("unknown");
                let color = if conclusion == "success" {
                    COLOR_SUCCESS
                } else {
                    COLOR_FAILURE
                };
                let thread_name = if conclusion == "success" {
                    "✅ CI Passed"
                } else {
                    "❌ CI Failed"
                };
                let name = e.workflow_run.name.as_deref().unwrap_or("CI");
                let branch = e.workflow_run.head_branch.as_deref().unwrap_or("unknown");

                (
                    thread_name,
                    format!("{} Run Details", name),
                    format!(
                        "**{}** - {}\nBranch: `{}`\n[View Run]({})",
                        name, conclusion, branch, e.workflow_run.html_url
                    ),
                    color,
                    Some(format!("Branch: {}", branch)),
                )
            }
            ParsedEvent::PullRequest(e) => {
                let has_bounty = e.pull_request.labels.iter().any(|l| l.name == "bounty");
                let color = if has_bounty { COLOR_BOUNTY } else { COLOR_PR };
                let thread_name = if has_bounty {
                    "🪙 PR with bounty"
                } else if e.action == "opened" {
                    "🧩 PR Opened"
                } else {
                    "🧩 PR Merged"
                };

                let action_verb = if e.action == "opened" {
                    "Opened"
                } else {
                    "Merged"
                };
                (
                    thread_name,
                    e.pull_request.title.clone(),
                    format!(
                        "{} by @{}\n[View PR]({})",
                        action_verb, e.sender.login, e.pull_request.html_url
                    ),
                    color,
                    Some(format!("by @{}", e.sender.login)),
                )
            }
            ParsedEvent::Issue(e) => {
                let has_bounty = e.issue.labels.iter().any(|l| l.name == "bounty");
                let color = if has_bounty {
                    COLOR_BOUNTY
                } else {
                    COLOR_ISSUE
                };
                let thread_name = if has_bounty {
                    "🪙 Issue with bounty"
                } else {
                    "📋 Other issues"
                };

                (
                    thread_name,
                    e.issue.title.clone(),
                    format!(
                        "Opened by @{}\n[View Issue]({})",
                        e.sender.login, e.issue.html_url
                    ),
                    color,
                    Some(format!("by @{}", e.sender.login)),
                )
            }
            ParsedEvent::Release(e) => (
                "🚀 Release Published",
                format!("Release {}", e.release.tag_name),
                format!(
                    "{}\n\n[View Release]({})",
                    e.release.body.as_deref().unwrap_or(""),
                    e.release.html_url
                ),
                COLOR_SUCCESS,
                Some(format!("by @{}", e.sender.login)),
            ),
            _ => return Ok(()),
        };

        // Reuse thread if it exists
        if let Some(tid) = self
            .discord
            .find_active_thread_by_name(guild_id, forum_id, thread_name)
            .await?
        {
            self.discord
                .send_message_with_embed(tid, &title, &description, color, footer_text.as_deref())
                .await?;
        } else {
            self.discord
                .create_forum_thread_with_embed(
                    forum_id,
                    thread_name,
                    &title,
                    &description,
                    color,
                    footer_text.as_deref(),
                )
                .await?;
        }

        Ok(())
    }

    async fn post_to_announcements(
        &self,
        event: &ParsedEvent,
        project: &projects::Project,
    ) -> Result<()> {
        if project.guild_id.is_empty() {
            return Ok(());
        }

        let guild_id = Id::new(project.guild_id.parse::<u64>().unwrap_or(0));
        let config =
            match crate::governance::server_config::get_config(&self.pool, &project.guild_id)
                .await?
            {
                Some(c) => c,
                None => return Ok(()),
            };

        // Self-healing for announcements channel
        let announce_channel = match self
            .discord
            .find_channel_by_name(guild_id, "announcements")
            .await?
        {
            Some(id) => {
                // Sync if DB has wrong ID
                if id.get().to_string() != config.announcements_id {
                    crate::governance::server_config::save_config(
                        &self.pool,
                        &project.guild_id,
                        &id.get().to_string(),
                        &config.github_forum_id,
                        config.mod_category_id.as_deref(),
                        config.project_review_id.as_deref(),
                        config.approvals_id.as_deref(),
                    )
                    .await?;
                }
                id
            }
            None => {
                let id = self.discord.create_announcements_channel(guild_id).await?;
                crate::governance::server_config::save_config(
                    &self.pool,
                    &project.guild_id,
                    &id.get().to_string(),
                    &config.github_forum_id,
                    config.mod_category_id.as_deref(),
                    config.project_review_id.as_deref(),
                    config.approvals_id.as_deref(),
                )
                .await?;
                id
            }
        };

        match event {
            ParsedEvent::Release(e) => {
                self.discord
                    .send_message_with_embed(
                        announce_channel,
                        &format!("🚀 New Release: {}", e.release.tag_name),
                        &format!(
                            "{}\n\n[View Release]({})",
                            e.release.body.as_deref().unwrap_or(""),
                            e.release.html_url
                        ),
                        COLOR_SUCCESS,
                        Some(&format!("Project: {}", project.name)),
                    )
                    .await?;
            }
            ParsedEvent::PullRequest(e) => {
                if e.pull_request.labels.iter().any(|l| l.name == "bounty") {
                    let verb = match e.action.as_str() {
                        "opened" => "Opened",
                        "labeled" => "Labeled",
                        _ => "Merged",
                    };
                    self.discord
                        .send_message_with_embed(
                            announce_channel,
                            &format!("🪙 Bounty PR {}: #{}", verb, e.pull_request.number),
                            &format!(
                                "**{}**\nby @{}\n[View PR]({})",
                                e.pull_request.title, e.sender.login, e.pull_request.html_url
                            ),
                            COLOR_BOUNTY,
                            Some(&format!("Project: {}", project.name)),
                        )
                        .await?;
                }
            }
            ParsedEvent::Issue(e) => {
                if e.issue.labels.iter().any(|l| l.name == "bounty") {
                    let verb = if e.action == "labeled" {
                        "Labeled"
                    } else {
                        "Opened"
                    };
                    self.discord
                        .send_message_with_embed(
                            announce_channel,
                            &format!("🪙 Bounty Issue {}: #{}", verb, e.issue.number),
                            &format!(
                                "**{}**\nby @{}\n[View Issue]({})",
                                e.issue.title, e.sender.login, e.issue.html_url
                            ),
                            COLOR_BOUNTY,
                            Some(&format!("Project: {}", project.name)),
                        )
                        .await?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}
