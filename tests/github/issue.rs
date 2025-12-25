use bytehub::discord::client::DiscordClient;
use bytehub::github::events::{Issue, IssueEvent, Label, ParsedEvent, Repository, User};
use bytehub::router::dispatch::Dispatcher;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test]
async fn test_issue_opened_triage(pool: PgPool) {
    let discord = Arc::new(DiscordClient::new("token", 123));
    let dispatcher = Dispatcher::new(pool, discord);

    let event = ParsedEvent::Issue(IssueEvent {
        action: "opened".into(),
        issue: Issue {
            number: 1,
            title: "Test Issue".into(),
            html_url: "http://github.com".into(),
            labels: vec![],
        },
        repository: Repository {
            full_name: "test/repo".into(),
            name: "repo".into(),
        },
        sender: User {
            login: "test-user".into(),
        },
    });

    // Issue opened should be logged and posted
    assert!(dispatcher.should_log(&event));
    assert!(dispatcher.should_post(&event));
    assert!(!dispatcher.should_announce(&event));
}

#[sqlx::test]
async fn test_issue_with_bounty_announcement(pool: PgPool) {
    let discord = Arc::new(DiscordClient::new("token", 123));
    let dispatcher = Dispatcher::new(pool, discord);

    let event = ParsedEvent::Issue(IssueEvent {
        action: "opened".into(),
        issue: Issue {
            number: 1,
            title: "Bounty Issue".into(),
            html_url: "http://github.com".into(),
            labels: vec![Label {
                name: "bounty".into(),
            }],
        },
        repository: Repository {
            full_name: "test/repo".into(),
            name: "repo".into(),
        },
        sender: User {
            login: "test-user".into(),
        },
    });

    // Issue with 'bounty' label should be announced
    assert!(dispatcher.should_announce(&event));
}

#[sqlx::test]
async fn test_issue_labeled_triage(pool: PgPool) {
    let discord = Arc::new(DiscordClient::new("token", 123));
    let dispatcher = Dispatcher::new(pool, discord);

    let event = ParsedEvent::Issue(IssueEvent {
        action: "labeled".into(),
        issue: Issue {
            number: 1,
            title: "Labeled Issue".into(),
            html_url: "http://github.com".into(),
            labels: vec![Label { name: "bug".into() }],
        },
        repository: Repository {
            full_name: "test/repo".into(),
            name: "repo".into(),
        },
        sender: User {
            login: "test-user".into(),
        },
    });

    // New labels should be logged and posted to thread
    assert!(dispatcher.should_log(&event));
    assert!(dispatcher.should_post(&event));
}
