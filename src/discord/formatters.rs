/// Colors for CI status
pub const COLOR_SUCCESS: u32 = 0x2ECC71; // Green
pub const COLOR_FAILURE: u32 = 0xE74C3C; // Red
pub const COLOR_SKIPPED: u32 = 0x95A5A6; // Grey
pub const COLOR_PR: u32 = 0x9B59B6; // Purple - PR merged
pub const COLOR_BOUNTY: u32 = 0xF1C40F; // Gold - Bounty events
pub const COLOR_ISSUE: u32 = 0x3498DB; // Blue - Other issues

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
