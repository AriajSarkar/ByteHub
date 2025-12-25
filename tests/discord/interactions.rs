use bytehub::discord::commands::{do_approve, do_setup_server, CommandOption, InteractionData};
use bytehub::governance::projects;
use bytehub::governance::server_config;
use sqlx::PgPool;
use twilight_model::guild::Permissions;

#[path = "../common/common.rs"]
mod common;

// Bits for: Manage Channels, Manage Threads, Send Messages, Embed Links, Read Message History
const REQUIRED_PERMS: Permissions = Permissions::from_bits_retain(326417599504);

#[sqlx::test]
async fn test_setup_server_success(pool: PgPool) {
    let state = common::create_state(pool.clone(), REQUIRED_PERMS);

    let guild_id = Some("123456".to_string());
    let result = do_setup_server(&state, &guild_id).await;

    assert!(result.is_ok());
    assert!(result.unwrap().contains("Server setup complete"));

    // Verify config saved in DB
    let config = server_config::get_config(&pool, "123456").await.unwrap();
    assert!(config.is_some());
    let config = config.unwrap();
    assert_eq!(config.announcements_id, "100");
}

#[sqlx::test]
async fn test_setup_server_permission_denied(pool: PgPool) {
    let state = common::create_state(pool.clone(), Permissions::empty());

    let guild_id = Some("123456".to_string());
    let result = do_setup_server(&state, &guild_id).await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Missing permissions"));
}

#[sqlx::test]
async fn test_approve_success(pool: PgPool) {
    let state = common::create_state(pool.clone(), REQUIRED_PERMS);

    // 1. Create a project pending approval
    let repo = "test/project";
    projects::submit_project(&pool, repo).await.unwrap();

    let guild_id = Some("123456".to_string());

    // Setup server first
    do_setup_server(&state, &guild_id).await.unwrap();

    // 2. Prepare interaction data
    let data = InteractionData {
        name: "approve".into(),
        options: Some(vec![CommandOption {
            name: "repo".into(),
            value: repo.into(),
        }]),
    };

    let result = do_approve(&state, &data, &guild_id).await;

    assert!(result.is_ok());
    assert!(result.unwrap().contains("approved"));

    // 3. Verify project is now approved in DB
    let project = projects::get_project(&pool, repo).await.unwrap().unwrap();
    assert!(project.is_approved);
    assert_eq!(project.forum_channel_id, "300");
}

#[sqlx::test]
async fn test_setup_server_no_guild(pool: PgPool) {
    let state = common::create_state(pool.clone(), REQUIRED_PERMS);

    // No guild ID provided (e.g. DM)
    let guild_id = None;
    let result = do_setup_server(&state, &guild_id).await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "invalid payload: missing guild_id"
    );
}

#[sqlx::test]
async fn test_approve_permission_denied(pool: PgPool) {
    let state = common::create_state(pool.clone(), Permissions::empty());

    let data = InteractionData {
        name: "approve".into(),
        options: Some(vec![CommandOption {
            name: "repo".into(),
            value: "test/repo".into(),
        }]),
    };

    let guild_id = Some("123456".to_string());
    let result = do_approve(&state, &data, &guild_id).await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Missing permissions"));
}

#[sqlx::test]
async fn test_approve_project_not_found(pool: PgPool) {
    let state = common::create_state(pool.clone(), REQUIRED_PERMS);

    let data = InteractionData {
        name: "approve".into(),
        options: Some(vec![CommandOption {
            name: "repo".into(),
            value: "nonexistent/repo".into(),
        }]),
    };

    let guild_id = Some("123456".to_string());

    // Setup server first
    do_setup_server(&state, &guild_id).await.unwrap();

    let result = do_approve(&state, &data, &guild_id).await;

    assert!(result.is_err());
    // Expecting error about project not found
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not found") || err.contains("no such project"));
}

#[sqlx::test]
async fn test_approve_already_approved(pool: PgPool) {
    let state = common::create_state(pool.clone(), REQUIRED_PERMS);

    // 1. Submit and approve the project first
    let repo = "test/verified_project";
    projects::submit_project(&pool, repo).await.unwrap();
    let _ = projects::approve_project(&pool, repo).await;

    // 2. Try to approve again
    let data = InteractionData {
        name: "approve".into(),
        options: Some(vec![CommandOption {
            name: "repo".into(),
            value: repo.into(),
        }]),
    };

    let guild_id = Some("123456".to_string());

    // Setup server first
    do_setup_server(&state, &guild_id).await.unwrap();

    let result = do_approve(&state, &data, &guild_id).await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "invalid payload: Project is already approved"
    );
}
