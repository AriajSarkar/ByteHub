use bytehub::discord::client::DiscordClient;
use bytehub::github::events::{Issue, IssueEvent, Label, ParsedEvent, Repository, User};
use bytehub::router::dispatch::Dispatcher;
use bytehub::storage::convex::ConvexDb;
use std::sync::Arc;

async fn create_test_dispatcher() -> Dispatcher {
    // Install crabgraph TLS provider for tests
    let _ = crabgraph::tls::try_install_default();

    dotenvy::dotenv().ok();
    let convex_url = std::env::var("CONVEX_URL").expect("CONVEX_URL required for tests");
    let db = ConvexDb::new(&convex_url)
        .await
        .expect("Failed to connect to Convex");
    let discord = Arc::new(DiscordClient::new("token", 123));
    Dispatcher::new(db, discord)
}

#[tokio::test]
async fn test_issue_opened_triage() {
    let dispatcher = create_test_dispatcher().await;

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

#[tokio::test]
async fn test_issue_with_bounty_announcement() {
    let dispatcher = create_test_dispatcher().await;

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

#[tokio::test]
async fn test_issue_labeled_triage() {
    let dispatcher = create_test_dispatcher().await;

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
