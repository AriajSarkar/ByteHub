use bytehub::discord::client::DiscordClient;
use bytehub::github::events::{ParsedEvent, Repository, User, WorkflowRun, WorkflowRunEvent};
use bytehub::router::dispatch::Dispatcher;
use bytehub::storage::convex::ConvexDb;
use std::sync::Arc;

async fn create_test_dispatcher() -> Dispatcher {
    crabgraph::tls::install_default();
    dotenvy::dotenv().ok();
    let convex_url = std::env::var("CONVEX_URL").expect("CONVEX_URL required for tests");
    let db = ConvexDb::new(&convex_url)
        .await
        .expect("Failed to connect to Convex");
    let discord = Arc::new(DiscordClient::new("token", 123));
    Dispatcher::new(db, discord)
}

#[tokio::test]
async fn test_workflow_success_triage() {
    let dispatcher = create_test_dispatcher().await;

    let event = ParsedEvent::WorkflowRun(WorkflowRunEvent {
        action: "completed".into(),
        workflow_run: WorkflowRun {
            id: 1,
            name: Some("CI".into()),
            conclusion: Some("success".into()),
            html_url: "http://github.com".into(),
            head_branch: Some("main".into()),
        },
        repository: Repository {
            full_name: "test/repo".into(),
            name: "repo".into(),
        },
        sender: User {
            login: "test-user".into(),
        },
    });

    // Successful main branch run should be logged and posted
    assert!(dispatcher.should_log(&event));
    assert!(dispatcher.should_post(&event));
}

#[tokio::test]
async fn test_workflow_failure_triage() {
    let dispatcher = create_test_dispatcher().await;

    let event = ParsedEvent::WorkflowRun(WorkflowRunEvent {
        action: "completed".into(),
        workflow_run: WorkflowRun {
            id: 1,
            name: Some("CI".into()),
            conclusion: Some("failure".into()),
            html_url: "http://github.com".into(),
            head_branch: Some("main".into()),
        },
        repository: Repository {
            full_name: "test/repo".into(),
            name: "repo".into(),
        },
        sender: User {
            login: "test-user".into(),
        },
    });

    // Failed main branch run should be logged and posted
    assert!(dispatcher.should_log(&event));
    assert!(dispatcher.should_post(&event));
}

#[tokio::test]
async fn test_workflow_in_progress_ignored() {
    let dispatcher = create_test_dispatcher().await;

    let event = ParsedEvent::WorkflowRun(WorkflowRunEvent {
        action: "requested".into(),
        workflow_run: WorkflowRun {
            id: 1,
            name: Some("CI".into()),
            conclusion: None,
            html_url: "http://github.com".into(),
            head_branch: Some("main".into()),
        },
        repository: Repository {
            full_name: "test/repo".into(),
            name: "repo".into(),
        },
        sender: User {
            login: "test-user".into(),
        },
    });

    // In-progress runs should NOT be logged or posted
    assert!(!dispatcher.should_log(&event));
    assert!(!dispatcher.should_post(&event));
}

#[tokio::test]
async fn test_workflow_feature_branch_ignored() {
    let dispatcher = create_test_dispatcher().await;

    let event = ParsedEvent::WorkflowRun(WorkflowRunEvent {
        action: "completed".into(),
        workflow_run: WorkflowRun {
            id: 1,
            name: Some("CI".into()),
            conclusion: Some("success".into()),
            html_url: "http://github.com".into(),
            head_branch: Some("feature/cool-stuff".into()),
        },
        repository: Repository {
            full_name: "test/repo".into(),
            name: "repo".into(),
        },
        sender: User {
            login: "test-user".into(),
        },
    });

    // Feature branch runs should be logged to activity but NOT posted to sidebar
    assert!(dispatcher.should_log(&event));
    assert!(!dispatcher.should_post(&event));
}
