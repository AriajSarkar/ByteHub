use bytehub::discord::client::DiscordClient;
use bytehub::github::events::{
    Label, ParsedEvent, PullRequest, PullRequestEvent, Repository, User,
};
use bytehub::router::dispatch::Dispatcher;
use bytehub::storage::convex::ConvexDb;
use std::sync::Arc;

async fn create_test_dispatcher() -> Dispatcher {
    dotenvy::dotenv().ok();
    let convex_url = std::env::var("CONVEX_URL").expect("CONVEX_URL required for tests");
    let db = ConvexDb::new(&convex_url)
        .await
        .expect("Failed to connect to Convex");
    let discord = Arc::new(DiscordClient::new("token", 123));
    Dispatcher::new(db, discord)
}

#[tokio::test]
async fn test_pr_opened_triage() {
    let dispatcher = create_test_dispatcher().await;

    let event = ParsedEvent::PullRequest(PullRequestEvent {
        action: "opened".into(),
        pull_request: PullRequest {
            number: 1,
            title: "Test PR".into(),
            html_url: "http://github.com".into(),
            merged: Some(false),
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

    // PR Opened should be logged and posted to thread
    assert!(dispatcher.should_log(&event));
    assert!(dispatcher.should_post(&event));
    assert!(!dispatcher.should_announce(&event));
}

#[tokio::test]
async fn test_pr_merged_triage() {
    let dispatcher = create_test_dispatcher().await;

    let event = ParsedEvent::PullRequest(PullRequestEvent {
        action: "closed".into(),
        pull_request: PullRequest {
            number: 1,
            title: "Test PR".into(),
            html_url: "http://github.com".into(),
            merged: Some(true),
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

    // PR Merged should be logged and posted
    assert!(dispatcher.should_log(&event));
    assert!(dispatcher.should_post(&event));
}

#[tokio::test]
async fn test_pr_with_bounty_announcement() {
    let dispatcher = create_test_dispatcher().await;

    let event = ParsedEvent::PullRequest(PullRequestEvent {
        action: "opened".into(),
        pull_request: PullRequest {
            number: 1,
            title: "Bounty PR".into(),
            html_url: "http://github.com".into(),
            merged: Some(false),
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

    // PR with 'bounty' label should be announced
    assert!(dispatcher.should_announce(&event));
}

#[tokio::test]
async fn test_bot_actor_exclusion() {
    let dispatcher = create_test_dispatcher().await;

    let event = ParsedEvent::PullRequest(PullRequestEvent {
        action: "opened".into(),
        pull_request: PullRequest {
            number: 1,
            title: "Bot PR".into(),
            html_url: "http://github.com".into(),
            merged: Some(false),
            labels: vec![],
        },
        repository: Repository {
            full_name: "test/repo".into(),
            name: "repo".into(),
        },
        sender: User {
            login: "dependabot[bot]".into(),
        },
    });

    // Bot actors should be recognized and potentially excluded from some posts
    assert!(dispatcher.is_bot_actor("dependabot[bot]"));
    assert!(!dispatcher.should_post(&event));
}
