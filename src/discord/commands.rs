use axum::{body::Bytes, extract::State, http::HeaderMap, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::warn;

use crate::discord::verify::verify_discord_signature;
use crate::error::{Error, Result};
use crate::governance::{projects, whitelist};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct Interaction {
    #[serde(rename = "type")]
    pub kind: u8,
    pub data: Option<InteractionData>,
    pub member: Option<Member>,
    pub token: String,
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct InteractionData {
    pub name: String,
    pub options: Option<Vec<CommandOption>>,
}

#[derive(Debug, Deserialize)]
pub struct CommandOption {
    pub name: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct Member {
    pub user: User,
    pub roles: Vec<String>,
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

        let response = match data.name.as_str() {
            "submit-project" => handle_submit_project(&state.pool, data).await?,
            "approve" => handle_approve(&state.pool, member, data).await?,
            "deny" => handle_deny(&state.pool, member, data).await?,
            "whitelist-user" => handle_whitelist(&state.pool, member, data).await?,
            "list" => handle_list(&state.pool, member).await?,
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

async fn handle_submit_project(pool: &PgPool, data: &InteractionData) -> Result<String> {
    let opts = data
        .options
        .as_ref()
        .ok_or(Error::InvalidPayload("missing options".into()))?;
    let repo = opts
        .iter()
        .find(|o| o.name == "repo")
        .and_then(|o| o.value.as_str())
        .ok_or(Error::InvalidPayload("missing repo".into()))?;
    let channel = opts
        .iter()
        .find(|o| o.name == "channel")
        .and_then(|o| o.value.as_str())
        .ok_or(Error::InvalidPayload("missing channel".into()))?;

    projects::submit_project(pool, repo, channel).await?;
    Ok(format!("Project `{}` submitted for approval.", repo))
}

async fn handle_approve(
    pool: &PgPool,
    member: Option<&Member>,
    data: &InteractionData,
) -> Result<String> {
    check_moderator(pool, member).await?;
    let opts = data
        .options
        .as_ref()
        .ok_or(Error::InvalidPayload("missing options".into()))?;
    let repo = opts
        .iter()
        .find(|o| o.name == "project")
        .and_then(|o| o.value.as_str())
        .ok_or(Error::InvalidPayload("missing project".into()))?;

    projects::approve_project(pool, repo).await?;
    Ok(format!("Project `{}` approved.", repo))
}

async fn handle_deny(
    pool: &PgPool,
    member: Option<&Member>,
    data: &InteractionData,
) -> Result<String> {
    check_moderator(pool, member).await?;
    let opts = data
        .options
        .as_ref()
        .ok_or(Error::InvalidPayload("missing options".into()))?;
    let repo = opts
        .iter()
        .find(|o| o.name == "project")
        .and_then(|o| o.value.as_str())
        .ok_or(Error::InvalidPayload("missing project".into()))?;

    projects::deny_project(pool, repo).await?;
    Ok(format!("Project `{}` denied and removed.", repo))
}

async fn handle_whitelist(
    pool: &PgPool,
    member: Option<&Member>,
    data: &InteractionData,
) -> Result<String> {
    check_moderator(pool, member).await?;
    let opts = data
        .options
        .as_ref()
        .ok_or(Error::InvalidPayload("missing options".into()))?;
    let username = opts
        .iter()
        .find(|o| o.name == "username")
        .and_then(|o| o.value.as_str())
        .ok_or(Error::InvalidPayload("missing username".into()))?;

    whitelist::add_user(pool, username).await?;
    Ok(format!("User `{}` added to whitelist.", username))
}

async fn check_moderator(pool: &PgPool, member: Option<&Member>) -> Result<()> {
    let member = member.ok_or(Error::Unauthorized)?;
    let is_mod = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM moderators WHERE discord_id = $1)",
    )
    .bind(&member.user.id)
    .fetch_one(pool)
    .await?;

    if !is_mod {
        return Err(Error::Unauthorized);
    }
    Ok(())
}

async fn handle_list(pool: &PgPool, member: Option<&Member>) -> Result<String> {
    check_moderator(pool, member).await?;

    let projects_list = projects::list_projects(pool).await?;

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
