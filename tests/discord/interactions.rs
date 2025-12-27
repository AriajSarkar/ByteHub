use bytehub::discord::commands::{do_approve, do_setup_server, CommandOption, InteractionData};
use bytehub::governance::projects;
use bytehub::governance::server_config;
use std::time::{SystemTime, UNIX_EPOCH};
use twilight_model::guild::Permissions;

#[path = "../common/common.rs"]
mod common;

// Bits for: Manage Channels, Manage Threads, Send Messages, Embed Links, Read Message History
const REQUIRED_PERMS: Permissions = Permissions::from_bits_retain(326417599504);

/// Generate a unique test name to avoid conflicts
fn unique_name(prefix: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("{}/{}", prefix, timestamp)
}

/// Generate a unique numeric guild ID
fn unique_guild_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    // Use timestamp as guild ID (must be valid u64)
    format!("{}", timestamp)
}

#[tokio::test]
async fn test_setup_server_success() {
    let state = common::create_state(REQUIRED_PERMS).await;

    let guild_id_str = unique_guild_id();
    let guild_id = Some(guild_id_str.clone());
    let result = do_setup_server(&state, &guild_id).await;

    assert!(result.is_ok());
    assert!(result.unwrap().contains("Server setup complete"));

    // Verify config saved in DB
    let config = server_config::get_config(&state.db, &guild_id_str)
        .await
        .unwrap();
    assert!(config.is_some());
    let config = config.unwrap();
    assert_eq!(config.announcements_id, "100");
}

#[tokio::test]
async fn test_setup_server_permission_denied() {
    let state = common::create_state(Permissions::empty()).await;

    let guild_id = Some(unique_guild_id());
    let result = do_setup_server(&state, &guild_id).await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Missing permissions"));
}

#[tokio::test]
async fn test_approve_success() {
    let state = common::create_state(REQUIRED_PERMS).await;

    // 1. Create a project pending approval with unique name
    let repo = unique_name("test_approve");
    projects::submit_project(&state.db, &repo).await.unwrap();

    let guild_id_str = unique_guild_id();
    let guild_id = Some(guild_id_str.clone());

    // Setup server first
    do_setup_server(&state, &guild_id).await.unwrap();

    // 2. Prepare interaction data
    let data = InteractionData {
        name: "approve".into(),
        options: Some(vec![CommandOption {
            name: "repo".into(),
            value: repo.clone().into(),
        }]),
    };

    let result = do_approve(&state, &data, &guild_id).await;

    assert!(result.is_ok());
    assert!(result.unwrap().contains("approved"));

    // 3. Verify project is now approved in DB
    let project = projects::get_project(&state.db, &repo)
        .await
        .unwrap()
        .unwrap();
    assert!(project.is_approved);
    assert_eq!(project.forum_channel_id, "300");
}

#[tokio::test]
async fn test_setup_server_no_guild() {
    let state = common::create_state(REQUIRED_PERMS).await;

    // No guild ID provided (e.g. DM)
    let guild_id = None;
    let result = do_setup_server(&state, &guild_id).await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "invalid payload: missing guild_id"
    );
}

#[tokio::test]
async fn test_approve_permission_denied() {
    let state = common::create_state(Permissions::empty()).await;

    let data = InteractionData {
        name: "approve".into(),
        options: Some(vec![CommandOption {
            name: "repo".into(),
            value: "test/repo".into(),
        }]),
    };

    let guild_id = Some(unique_guild_id());
    let result = do_approve(&state, &data, &guild_id).await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Missing permissions"));
}

#[tokio::test]
async fn test_approve_project_not_found() {
    let state = common::create_state(REQUIRED_PERMS).await;

    // Use a unique non-existent project name
    let data = InteractionData {
        name: "approve".into(),
        options: Some(vec![CommandOption {
            name: "repo".into(),
            value: unique_name("nonexistent").into(),
        }]),
    };

    let guild_id_str = unique_guild_id();
    let guild_id = Some(guild_id_str.clone());

    // Setup server first
    do_setup_server(&state, &guild_id).await.unwrap();

    let result = do_approve(&state, &data, &guild_id).await;

    assert!(result.is_err());
    // Expecting error about project not found
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not found") || err.contains("Project not found"));
}

#[tokio::test]
async fn test_approve_already_approved() {
    let state = common::create_state(REQUIRED_PERMS).await;

    // 1. Submit and approve the project first with unique name
    let repo = unique_name("test_already_approved");
    projects::submit_project(&state.db, &repo).await.unwrap();
    let _ = projects::approve_project(&state.db, &repo).await;

    // 2. Try to approve again via the command
    let data = InteractionData {
        name: "approve".into(),
        options: Some(vec![CommandOption {
            name: "repo".into(),
            value: repo.clone().into(),
        }]),
    };

    let guild_id_str = unique_guild_id();
    let guild_id = Some(guild_id_str.clone());

    // Setup server first
    do_setup_server(&state, &guild_id).await.unwrap();

    let result = do_approve(&state, &data, &guild_id).await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "invalid payload: Project is already approved"
    );
}
