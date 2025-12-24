use sqlx::PgPool;
use tracing::info;
use twilight_model::id::Id;

use crate::discord::client::DiscordClient;
use crate::discord::forum::{COLOR_BOUNTY, COLOR_FAILURE, COLOR_ISSUE, COLOR_PR, COLOR_SUCCESS};
use crate::error::Result;
use crate::github::events::{
    IssueEvent, ParsedEvent, PullRequestEvent, ReleaseEvent, WorkflowRunEvent,
};
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
            Some(r) => r,
            None => return Ok(()),
        };

        let project = match projects::get_approved_project(&self.pool, repo).await? {
            Some(p) => p,
            None => {
                info!(repo, "event from unlisted/unapproved project, ignoring");
                return Ok(());
            }
        };

        // Apply smart filtering
        if !self.should_post(&event) {
            info!(repo, "event filtered out by smart filtering");
            return Ok(());
        }

        // Get or create the project's activity thread
        let thread_id = self.get_or_create_thread(&project, repo).await?;
        let thread_channel = Id::new(thread_id.parse::<u64>().unwrap_or(0));

        // Post event to thread with colored embed
        self.post_event_to_thread(thread_channel, &event).await?;
        info!(repo, "posted event to project thread");

        // Check if should also post to announcements
        if self.should_announce(&event) {
            self.post_to_announcements(&event, repo).await?;
        }

        Ok(())
    }

    /// Smart filtering logic
    fn should_post(&self, event: &ParsedEvent) -> bool {
        match event {
            ParsedEvent::WorkflowRun(e) => {
                // Skip cancelled/skipped CI
                let conclusion = e.workflow_run.conclusion.as_deref().unwrap_or("unknown");
                if conclusion == "skipped" || conclusion == "cancelled" {
                    return false;
                }
                // Only main/master branch for CI
                let branch = e.workflow_run.head_branch.as_deref().unwrap_or("");
                branch == "main" || branch == "master"
            }
            ParsedEvent::PullRequest(e) => {
                // Skip bots (dependabot, renovate, etc.)
                if self.is_bot_actor(e.sender.login.as_str()) {
                    return false;
                }
                // Only merged PRs
                e.action == "closed" && e.pull_request.merged.unwrap_or(false)
            }
            ParsedEvent::Issue(_) => true,
            ParsedEvent::Release(_) => true,
            ParsedEvent::Unknown => false,
        }
    }

    /// Check if event should go to announcements
    fn should_announce(&self, event: &ParsedEvent) -> bool {
        match event {
            ParsedEvent::Release(_) => true,
            ParsedEvent::Issue(e) => e.issue.labels.iter().any(|l| l.name == "bounty"),
            ParsedEvent::PullRequest(e) => e.pull_request.labels.iter().any(|l| l.name == "bounty"),
            _ => false,
        }
    }

    fn is_bot_actor(&self, login: &str) -> bool {
        let bots = [
            "dependabot",
            "dependabot[bot]",
            "renovate",
            "renovate[bot]",
            "github-actions",
            "github-actions[bot]",
        ];
        bots.iter().any(|b| login.to_lowercase().contains(b))
    }

    async fn get_or_create_thread(
        &self,
        project: &projects::Project,
        repo: &str,
    ) -> Result<String> {
        // If thread already exists, use it
        if let Some(ref tid) = project.thread_id {
            if !tid.is_empty() {
                return Ok(tid.clone());
            }
        }

        // Create new thread in the project's forum channel
        let forum_id = Id::new(project.forum_channel_id.parse::<u64>().unwrap_or(0));
        let project_name = repo.split('/').last().unwrap_or(repo);
        let thread_name = format!("📦 {} Activity", project_name);

        // Create the initial thread
        let tid = self
            .discord
            .create_forum_thread(
                forum_id,
                &thread_name,
                "Project activity thread. All events will be posted here.",
            )
            .await?;

        // Save the thread ID to the database
        let tid_str = tid.get().to_string();
        projects::update_thread_id(&self.pool, repo, &tid_str).await?;

        Ok(tid_str)
    }

    async fn post_event_to_thread(
        &self,
        thread_id: twilight_model::id::Id<twilight_model::id::marker::ChannelMarker>,
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

                self.discord
                    .send_message_with_embed(
                        thread_id,
                        &format!("{} PR #{} merged", emoji, e.pull_request.number),
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

                self.discord
                    .send_message_with_embed(
                        thread_id,
                        &format!("{} Issue #{} opened", emoji, e.issue.number),
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

    async fn post_to_announcements(&self, event: &ParsedEvent, repo: &str) -> Result<()> {
        // TODO: Implement actual announcement posting
        // Would need to get announcements channel from server_config
        info!(repo, "would post to announcements");
        Ok(())
    }
}
