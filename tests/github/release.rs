use bytehub::discord::client::DiscordClient;
use bytehub::github::events::{ParsedEvent, Release, ReleaseEvent, Repository, User};
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
async fn test_release_published_triage() {
    let dispatcher = create_test_dispatcher().await;

    let event = ParsedEvent::Release(ReleaseEvent {
        action: "published".into(),
        release: Release {
            tag_name: "v1.0.0".into(),
            name: Some("Initial Release".into()),
            body: Some("Description".into()),
            html_url: "http://github.com".into(),
        },
        repository: Repository {
            full_name: "test/repo".into(),
            name: "repo".into(),
        },
        sender: User {
            login: "test-user".into(),
        },
    });

    // Published releases should be logged, posted, and announced
    assert!(dispatcher.should_log(&event));
    assert!(dispatcher.should_post(&event));
    assert!(dispatcher.should_announce(&event));
}
