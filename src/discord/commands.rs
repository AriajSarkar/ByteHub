use axum::{body::Bytes, extract::State, http::HeaderMap, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tracing::warn;

use crate::discord::rate_limit::RateLimiter;
use crate::discord::verify::verify_discord_signature;
use crate::error::{Error, Result};
use crate::governance::{projects, server_config, whitelist};
use crate::storage::convex::ConvexDb;
use crate::AppState;

use twilight_model::guild::Permissions;
use twilight_model::id::Id;

const REQUIRED_PERMISSIONS: Permissions = Permissions::from_bits_retain(326417599504);

/// Rate limiter for expensive commands (setup-server, approve)
/// 5 requests per 60 seconds per guild to prevent spam and database conflicts
fn get_rate_limiter() -> &'static RateLimiter {
    static RATE_LIMITER: OnceLock<RateLimiter> = OnceLock::new();
    RATE_LIMITER.get_or_init(|| RateLimiter::new(60, 5))
}

#[derive(Debug, Deserialize)]
pub struct Interaction {
    #[serde(rename = "type")]
    pub kind: u8,
    pub data: Option<InteractionData>,
    pub member: Option<Member>,
    pub guild_id: Option<String>,
    #[allow(dead_code)]
    pub token: String,
    #[allow(dead_code)]
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InteractionData {
    pub name: String,
    pub options: Option<Vec<CommandOption>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommandOption {
    pub name: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct Member {
    pub user: User,
    #[allow(dead_code)]
    pub roles: Vec<String>,
    pub permissions: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct InteractionResponse {
    #[serde(rename = "type")]
    pub kind: u8,
    pub data: Option<ResponseData>,
}

#[derive(Debug, Serialize)]
pub struct ResponseData {
    pub content: String,
    pub flags: Option<u32>,
}

pub async fn handle_interaction(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse> {
    let signature = headers
        .get("x-signature-ed25519")
        .and_then(|v| v.to_str().ok())
        .ok_or(Error::InvalidSignature)?;
    let timestamp = headers
        .get("x-signature-timestamp")
        .and_then(|v| v.to_str().ok())
        .ok_or(Error::InvalidSignature)?;

    if !verify_discord_signature(
        &state.config.discord_public_key,
        timestamp,
        &body,
        signature,
    ) {
        warn!("invalid discord signature");
        return Err(Error::InvalidSignature);
    }

    let interaction: Interaction =
        serde_json::from_slice(&body).map_err(|e| Error::InvalidPayload(e.to_string()))?;

    // Type 1 = PING
    if interaction.kind == 1 {
        return Ok(Json(InteractionResponse {
            kind: 1,
            data: None,
        }));
    }

    // Type 2 = APPLICATION_COMMAND
    if interaction.kind == 2 {
        let data = interaction
            .data
            .as_ref()
            .ok_or(Error::InvalidPayload("missing data".into()))?;
        let member = interaction.member.as_ref();

        // Commands that need deferred response (channel creation takes >3s)
        if data.name == "setup-server" || data.name == "approve" {
            // Defense-in-depth: Early-exit for DM invocations
            // (Commands should already be guild-only via dm_permission: false)
            if interaction.guild_id.is_none() {
                return Ok(Json(InteractionResponse {
                    kind: 4,
                    data: Some(ResponseData {
                        content: "❌ This command can only be used in a server.".to_string(),
                        flags: Some(64),
                    }),
                }));
            }

            // Rate limiting: Prevent command spam that causes database conflicts
            if let Some(ref gid) = interaction.guild_id {
                if let Err(wait_secs) = get_rate_limiter().check(gid) {
                    return Ok(Json(InteractionResponse {
                        kind: 4,
                        data: Some(ResponseData {
                            content: format!(
                                "⏳ Rate limited. Please wait {} seconds before running this command again.",
                                wait_secs
                            ),
                            flags: Some(64),
                        }),
                    }));
                }
            }

            check_moderator(member)?;
            let cmd_name = data.name.clone();
            let guild_id = interaction.guild_id.clone();
            let token = interaction.token.clone();
            let app_id = state.discord.application_id();
            let state_clone = state.clone();
            let data_clone = data.clone();

            tokio::spawn(async move {
                let result = match cmd_name.as_str() {
                    "setup-server" => do_setup_server(&state_clone, &guild_id).await,
                    "approve" => do_approve(&state_clone, &data_clone, &guild_id).await,
                    _ => Ok("Unknown".to_string()),
                };

                let content = match result {
                    Ok(msg) => msg,
                    Err(e) => format!("❌ Error: {}", e),
                };

                let url = format!("https://discord.com/api/v10/webhooks/{}/{}", app_id, token);
                let _ = reqwest::Client::new()
                    .post(&url)
                    .json(&serde_json::json!({ "content": content, "flags": 64 }))
                    .send()
                    .await;
            });

            return Ok(Json(InteractionResponse {
                kind: 5, // DEFERRED_CHANNEL_MESSAGE_WITH_SOURCE
                data: None,
            }));
        }

        let response = match data.name.as_str() {
            "submit-project" => handle_submit_project(&state.db, data).await?,
            "deny" => handle_deny(&state.db, member, data).await?,
            "whitelist-user" => handle_whitelist(&state.db, member, data).await?,
            "list" => handle_list(&state.db, member).await?,
            _ => "Unknown command".to_string(),
        };

        return Ok(Json(InteractionResponse {
            kind: 4,
            data: Some(ResponseData {
                content: response,
                flags: Some(64), // Ephemeral
            }),
        }));
    }

    Ok(Json(InteractionResponse {
        kind: 1,
        data: None,
    }))
}

async fn handle_submit_project(db: &ConvexDb, data: &InteractionData) -> Result<String> {
    let opts = data
        .options
        .as_ref()
        .ok_or(Error::InvalidPayload("missing options".into()))?;
    let repo = opts
        .iter()
        .find(|o| o.name == "repo")
        .and_then(|o| o.value.as_str())
        .ok_or(Error::InvalidPayload("missing repo".into()))?;

    // Handle the case where project already exists
    match projects::submit_project(db, repo).await {
        Ok(_) => Ok(format!("Project `{}` submitted for approval.", repo)),
        Err(Error::InvalidPayload(msg)) if msg.contains("already exists") => {
            Ok(format!("⚠️ Project `{}` has already been submitted.", repo))
        }
        Err(e) => Err(e),
    }
}

pub async fn do_approve(
    state: &AppState,
    data: &InteractionData,
    guild_id: &Option<String>,
) -> Result<String> {
    let opts = data
        .options
        .as_ref()
        .ok_or(Error::InvalidPayload("missing options".into()))?;
    let repo = opts
        .iter()
        .find(|o| o.name == "repo")
        .and_then(|o| o.value.as_str())
        .ok_or(Error::InvalidPayload("missing repo".into()))?;

    let guild_id_str = guild_id
        .as_ref()
        .ok_or(Error::InvalidPayload("missing guild_id".into()))?;

    let guild_id_u64: u64 = guild_id_str
        .parse()
        .map_err(|_| Error::InvalidPayload("invalid guild_id".into()))?;
    let gid = Id::new(guild_id_u64);

    // Verify permissions first
    let perms = state.discord.get_self_permissions(gid).await?;
    if !perms.contains(REQUIRED_PERMISSIONS) {
        let missing = REQUIRED_PERMISSIONS - perms;
        let invite_msg = match &state.config.discord_invite {
            Some(url) => format!(
                "\n\nUse this link to re-invite me with correct permissions: {}",
                url
            ),
            None => "".into(),
        };
        return Err(Error::Discord(format!(
            "Missing permissions: `{:?}`. Please update my role.{}",
            missing, invite_msg
        )));
    }

    // Get server config to find the GitHub category
    let config = server_config::get_config(&state.db, guild_id_str)
        .await?
        .ok_or(Error::InvalidPayload(
            "Server not set up. Run /setup-server first.".into(),
        ))?;

    // Find or create GitHub category (handles deleted/stale channels)
    // Note: save_config is idempotent, but we still avoid redundant calls for efficiency
    let github_category = match state.discord.find_channel_by_name(gid, "GitHub").await? {
        Some(id) => {
            // Category exists - use it directly
            // Config sync will happen via setup-server if needed
            id
        }
        None => {
            // Category missing - create and update config
            let id = state.discord.create_github_category(gid).await?;
            server_config::save_config(
                &state.db,
                guild_id_str,
                &config.announcements_id,
                &id.get().to_string(),
                config.mod_category_id.as_deref(),
                config.project_review_id.as_deref(),
                config.approvals_id.as_deref(),
            )
            .await?;
            id
        }
    };

    // Extract project name from repo (e.g., "AriajSarkar/eventix" -> "eventix")
    let project_name = repo.rsplit('/').next().unwrap_or(repo);

    // Check if project already has a forum channel and if it still exists in Discord
    let channels = state.discord.guild_channels(gid).await?;

    let existing_project = projects::get_project(&state.db, repo).await?;

    if let Some(p) = &existing_project {
        if p.is_approved {
            return Err(Error::InvalidPayload("Project is already approved".into()));
        }
    }

    let (project_forum_id, is_new) = if let Some(p) = existing_project {
        let mut found_id = None;
        if !p.forum_channel_id.is_empty() {
            if let Ok(id_u64) = p.forum_channel_id.parse::<u64>() {
                let id = twilight_model::id::Id::new(id_u64);
                // Verify it exists in the channel list
                if channels.iter().any(|c| c.id == id) {
                    found_id = Some(id);
                }
            }
        }

        if let Some(id) = found_id {
            (id, false)
        } else {
            (
                state
                    .discord
                    .create_project_forum(gid, github_category, project_name)
                    .await?,
                true,
            )
        }
    } else {
        (
            state
                .discord
                .create_project_forum(gid, github_category, project_name)
                .await?,
            true,
        )
    };

    // Update project with the forum channel ID and approve
    projects::approve_project_with_forum(
        &state.db,
        repo,
        &project_forum_id.get().to_string(),
        guild_id_str,
    )
    .await?;

    let action_msg = if is_new {
        format!("Created forum: <#{}>", project_forum_id)
    } else {
        format!("Reusing existing forum: <#{}>", project_forum_id)
    };

    Ok(format!("✅ Project `{}` approved!\n\n{}", repo, action_msg))
}

async fn handle_deny(
    db: &ConvexDb,
    member: Option<&Member>,
    data: &InteractionData,
) -> Result<String> {
    check_moderator(member)?;
    let opts = data
        .options
        .as_ref()
        .ok_or(Error::InvalidPayload("missing options".into()))?;
    let repo = opts
        .iter()
        .find(|o| o.name == "repo")
        .and_then(|o| o.value.as_str())
        .ok_or(Error::InvalidPayload("missing repo".into()))?;

    projects::deny_project(db, repo).await?;
    Ok(format!("Project `{}` denied and removed.", repo))
}

async fn handle_whitelist(
    db: &ConvexDb,
    member: Option<&Member>,
    data: &InteractionData,
) -> Result<String> {
    check_moderator(member)?;
    let opts = data
        .options
        .as_ref()
        .ok_or(Error::InvalidPayload("missing options".into()))?;
    let username = opts
        .iter()
        .find(|o| o.name == "username")
        .and_then(|o| o.value.as_str())
        .ok_or(Error::InvalidPayload("missing username".into()))?;

    whitelist::add_user(db, username).await?;
    Ok(format!("User `{}` added to whitelist.", username))
}

// Check if member has ADMINISTRATOR (0x8) or MANAGE_GUILD (0x20) permission
fn check_moderator(member: Option<&Member>) -> Result<()> {
    let member = member.ok_or(Error::Unauthorized)?;

    // Parse Discord permission bitfield
    let permissions: u64 = member
        .permissions
        .as_ref()
        .and_then(|p| p.parse().ok())
        .unwrap_or(0);

    const ADMINISTRATOR: u64 = 0x8;
    const MANAGE_GUILD: u64 = 0x20;

    if permissions & ADMINISTRATOR != 0 || permissions & MANAGE_GUILD != 0 {
        return Ok(());
    }

    Err(Error::Unauthorized)
}

async fn handle_list(db: &ConvexDb, member: Option<&Member>) -> Result<String> {
    check_moderator(member)?;

    let projects_list = projects::list_projects(db).await?;

    if projects_list.is_empty() {
        return Ok("No projects registered.".to_string());
    }

    let mut approved = Vec::new();
    let mut pending = Vec::new();

    for p in projects_list {
        let line = format!("• `{}`", p.github_repo);
        if p.is_approved {
            approved.push(line);
        } else {
            pending.push(line);
        }
    }

    let mut response = String::new();

    if !approved.is_empty() {
        response.push_str("**✅ Approved:**\n");
        response.push_str(&approved.join("\n"));
    }

    if !pending.is_empty() {
        if !response.is_empty() {
            response.push_str("\n\n");
        }
        response.push_str("**⏳ Pending:**\n");
        response.push_str(&pending.join("\n"));
    }

    Ok(response)
}

pub async fn do_setup_server(state: &AppState, guild_id: &Option<String>) -> Result<String> {
    let guild_id_str = guild_id
        .as_ref()
        .ok_or(Error::InvalidPayload("missing guild_id".into()))?;

    let guild_id_u64: u64 = guild_id_str
        .parse()
        .map_err(|_| Error::InvalidPayload("invalid guild_id".into()))?;

    let gid = Id::new(guild_id_u64);

    // Verify permissions first
    let perms = state.discord.get_self_permissions(gid).await?;
    if !perms.contains(REQUIRED_PERMISSIONS) {
        let missing = REQUIRED_PERMISSIONS - perms;
        let invite_msg = match &state.config.discord_invite {
            Some(url) => format!(
                "\n\nUse this link to re-invite me with correct permissions: {}",
                url
            ),
            None => "".into(),
        };
        return Err(Error::Discord(format!(
            "Missing permissions: `{:?}`. Please update my role or re-invite me.{}",
            missing, invite_msg
        )));
    }

    // Always find or create channels (handles deleted/stale channels)
    let announcements_id = match state
        .discord
        .find_channel_containing(gid, "announcements")
        .await?
    {
        Some(id) => id,
        None => state.discord.create_announcements_channel(gid).await?,
    };

    // Find or create GitHub category (container for project forums)
    let github_category_id = match state.discord.find_channel_by_name(gid, "GitHub").await? {
        Some(id) => id,
        None => state.discord.create_github_category(gid).await?,
    };

    // Find or create Mod category with channels
    // Use find_channel_containing to match categories like "🔒 Mod" or "Moderators"
    let (mod_cat_id, review_id, approvals_id) =
        match state.discord.find_channel_containing(gid, "mod").await? {
            Some(cat_id) => {
                // Category exists, find or create sub-channels
                let review = match state
                    .discord
                    .find_channel_by_name(gid, "project-review")
                    .await?
                {
                    Some(id) => id,
                    None => {
                        state
                            .discord
                            .create_channel_in_category(gid, cat_id, "project-review")
                            .await?
                    }
                };
                let approvals = match state.discord.find_channel_by_name(gid, "approvals").await? {
                    Some(id) => id,
                    None => {
                        state
                            .discord
                            .create_channel_in_category(gid, cat_id, "approvals")
                            .await?
                    }
                };
                (cat_id, review, approvals)
            }
            None => state.discord.create_mod_category(gid).await?,
        };

    // Save config to database
    server_config::save_config(
        &state.db,
        guild_id_str,
        &announcements_id.get().to_string(),
        &github_category_id.get().to_string(),
        Some(&mod_cat_id.get().to_string()),
        Some(&review_id.get().to_string()),
        Some(&approvals_id.get().to_string()),
    )
    .await?;

    Ok(format!(
        "✅ **Server setup complete!**\n\n**Created channels:**\n• <#{}> - Announcements\n• <#{}> - GitHub (Category)\n• <#{}> - Mod (project-review)\n• <#{}> - Mod (approvals)",
        announcements_id, github_category_id, review_id, approvals_id
    ))
}
