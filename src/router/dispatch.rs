use sqlx::PgPool;
use tracing::info;
use twilight_model::id::Id;

use crate::discord::client::DiscordClient;
use crate::discord::forum::{format_issue, format_pr_merged, format_release, format_workflow};
use crate::error::Result;
use crate::github::events::ParsedEvent;
use crate::governance::{projects, rules};

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

        let rule_match = match rules::evaluate_rules(&self.pool, project.id, &event).await? {
            Some(m) => m,
            None => {
                info!(repo, "no matching rule for event");
                return Ok(());
            }
        };

        let (title, body) = self.format_event(&event);

        if rule_match.actions.post_forum {
            let channel_id = Id::new(project.forum_channel_id.parse::<u64>().unwrap_or(0));
            self.discord
                .create_forum_thread(channel_id, &title, &body)
                .await?;
            info!(repo, rule_id = ?rule_match.rule_id, "posted to forum");
        }

        if rule_match.actions.post_announce {
            // announcements channel would be configured per-guild
            // for now we log it
            info!(repo, "would post to announcements: {}", title);
        }

        Ok(())
    }

    fn format_event(&self, event: &ParsedEvent) -> (String, String) {
        match event {
            ParsedEvent::Release(e) => format_release(e),
            ParsedEvent::PullRequest(e) => format_pr_merged(e),
            ParsedEvent::Issue(e) => format_issue(e),
            ParsedEvent::WorkflowRun(e) => format_workflow(e),
            ParsedEvent::Unknown => ("Unknown".into(), "Unknown event".into()),
        }
    }
}
