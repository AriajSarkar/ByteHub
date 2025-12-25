use bytehub::discord::client::DiscordClient;
use bytehub::github::events::{ParsedEvent, Release, ReleaseEvent, Repository, User};
use bytehub::router::dispatch::Dispatcher;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test]
async fn test_release_published_triage(pool: PgPool) {
    let discord = Arc::new(DiscordClient::new("token", 123));
    let dispatcher = Dispatcher::new(pool, discord);

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
