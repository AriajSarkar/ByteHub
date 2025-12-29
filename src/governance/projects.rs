use convex::Value as ConvexValue;
use maplit::btreemap;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::storage::convex::ConvexDb;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub github_repo: String,
    pub forum_channel_id: String,
    pub thread_id: Option<String>,
    pub guild_id: String,
    pub is_approved: bool,
}

/// Parse mutation result that returns { success: true/false, id?, error? }
fn parse_mutation_result(result: &serde_json::Value) -> Result<Option<String>> {
    let success = result
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if success {
        let id = result
            .get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        Ok(id)
    } else {
        let error = result
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error");
        // Return dedicated error variant for type-safe matching (case-insensitive)
        if error.to_lowercase().contains("already exists") {
            Err(Error::ProjectAlreadyExists(error.to_string()))
        } else {
            Err(Error::InvalidPayload(error.to_string()))
        }
    }
}

pub async fn submit_project(db: &ConvexDb, github_repo: &str) -> Result<String> {
    let result = db
        .mutation(
            "projects:submit",
            btreemap! {
                "github_repo".into() => ConvexValue::String(github_repo.to_string()),
            },
        )
        .await?;

    parse_mutation_result(&result)?
        .ok_or_else(|| Error::InvalidPayload("Expected ID from submit".into()))
}

pub async fn approve_project(db: &ConvexDb, github_repo: &str) -> Result<()> {
    let result = db
        .mutation(
            "projects:approve",
            btreemap! {
                "github_repo".into() => ConvexValue::String(github_repo.to_string()),
            },
        )
        .await?;

    parse_mutation_result(&result)?;
    Ok(())
}

pub async fn approve_project_with_forum(
    db: &ConvexDb,
    github_repo: &str,
    forum_channel_id: &str,
    guild_id: &str,
) -> Result<()> {
    let result = db
        .mutation(
            "projects:approveWithForum",
            btreemap! {
                "github_repo".into() => ConvexValue::String(github_repo.to_string()),
                "forum_channel_id".into() => ConvexValue::String(forum_channel_id.to_string()),
                "guild_id".into() => ConvexValue::String(guild_id.to_string()),
            },
        )
        .await?;

    parse_mutation_result(&result)?;
    Ok(())
}

pub async fn deny_project(db: &ConvexDb, github_repo: &str) -> Result<()> {
    let result = db
        .mutation(
            "projects:deny",
            btreemap! {
                "github_repo".into() => ConvexValue::String(github_repo.to_string()),
            },
        )
        .await?;

    // deny now returns { success: true/false }
    let success = result
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !success {
        let error = result
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("Project not found");
        return Err(Error::NotFound(error.to_string()));
    }

    Ok(())
}

pub async fn get_approved_project(db: &ConvexDb, github_repo: &str) -> Result<Option<Project>> {
    let result = db
        .query(
            "projects:getApproved",
            btreemap! {
                "github_repo".into() => ConvexValue::String(github_repo.to_string()),
            },
        )
        .await?;

    if result.is_null() {
        return Ok(None);
    }

    let project: Project = serde_json::from_value(result)
        .map_err(|e| Error::InvalidPayload(format!("Failed to parse project: {}", e)))?;

    Ok(Some(project))
}

pub async fn get_project(db: &ConvexDb, github_repo: &str) -> Result<Option<Project>> {
    let result = db
        .query(
            "projects:get",
            btreemap! {
                "github_repo".into() => ConvexValue::String(github_repo.to_string()),
            },
        )
        .await?;

    if result.is_null() {
        return Ok(None);
    }

    let project: Project = serde_json::from_value(result)
        .map_err(|e| Error::InvalidPayload(format!("Failed to parse project: {}", e)))?;

    Ok(Some(project))
}

pub async fn list_projects_by_guild(db: &ConvexDb, guild_id: &str) -> Result<Vec<Project>> {
    let result = db
        .query(
            "projects:listByGuild",
            btreemap! {
                "guild_id".into() => ConvexValue::String(guild_id.to_string()),
            },
        )
        .await?;

    let projects: Vec<Project> = serde_json::from_value(result)
        .map_err(|e| Error::InvalidPayload(format!("Failed to parse projects: {}", e)))?;

    Ok(projects)
}

pub async fn update_forum_id(db: &ConvexDb, repo: &str, forum_id: &str) -> Result<()> {
    db.mutation(
        "projects:updateForumId",
        btreemap! {
            "github_repo".into() => ConvexValue::String(repo.to_string()),
            "forum_id".into() => ConvexValue::String(forum_id.to_string()),
        },
    )
    .await?;

    Ok(())
}

pub async fn update_thread_id(db: &ConvexDb, repo: &str, thread_id: &str) -> Result<()> {
    db.mutation(
        "projects:updateThreadId",
        btreemap! {
            "github_repo".into() => ConvexValue::String(repo.to_string()),
            "thread_id".into() => ConvexValue::String(thread_id.to_string()),
        },
    )
    .await?;

    Ok(())
}
