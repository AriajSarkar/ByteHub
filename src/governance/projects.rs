use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{Error, Result};

#[derive(Debug, sqlx::FromRow)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub github_repo: String,
    pub forum_channel_id: String,
    pub thread_id: Option<String>,
    pub is_approved: bool,
}

pub async fn submit_project(pool: &PgPool, github_repo: &str) -> Result<Uuid> {
    let name = github_repo.split('/').last().unwrap_or(github_repo);
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO projects (name, github_repo, forum_channel_id)
        VALUES ($1, $2, '')
        ON CONFLICT (github_repo) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(name)
    .bind(github_repo)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| Error::InvalidPayload("project already exists".into()))?;

    Ok(id)
}

pub async fn approve_project(pool: &PgPool, github_repo: &str) -> Result<()> {
    let rows = sqlx::query("UPDATE projects SET is_approved = true WHERE github_repo = $1")
        .bind(github_repo)
        .execute(pool)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(Error::NotFound("project not found".into()));
    }
    Ok(())
}

pub async fn approve_project_with_forum(
    pool: &PgPool,
    github_repo: &str,
    forum_channel_id: &str,
) -> Result<()> {
    // Get project ID first
    let project_id: Option<uuid::Uuid> =
        sqlx::query_scalar("SELECT id FROM projects WHERE github_repo = $1")
            .bind(github_repo)
            .fetch_optional(pool)
            .await?;

    let project_id = project_id.ok_or(Error::NotFound("project not found".into()))?;

    // Update project
    sqlx::query(
        "UPDATE projects SET is_approved = true, forum_channel_id = $2 WHERE github_repo = $1",
    )
    .bind(github_repo)
    .bind(forum_channel_id)
    .execute(pool)
    .await?;

    // Create default rules for this project (catch-all for all event types)
    let default_rules = vec![
        // Workflow runs - post all
        (
            r#"{"event_type": "workflow_run.completed"}"#,
            r#"{"post_forum": true, "post_announce": false}"#,
        ),
        // Releases - post all
        (
            r#"{"event_type": "release.published"}"#,
            r#"{"post_forum": true, "post_announce": true}"#,
        ),
        // PRs merged - post all
        (
            r#"{"event_type": "pull_request.closed", "merged": true}"#,
            r#"{"post_forum": true, "post_announce": false}"#,
        ),
        // Issues opened - post all
        (
            r#"{"event_type": "issues.opened"}"#,
            r#"{"post_forum": true, "post_announce": false}"#,
        ),
    ];

    for (i, (conditions, actions)) in default_rules.iter().enumerate() {
        sqlx::query(
            "INSERT INTO rules (project_id, priority, conditions, actions) VALUES ($1, $2, $3::jsonb, $4::jsonb) ON CONFLICT DO NOTHING"
        )
        .bind(project_id)
        .bind(i as i32)
        .bind(conditions)
        .bind(actions)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn deny_project(pool: &PgPool, github_repo: &str) -> Result<()> {
    sqlx::query("DELETE FROM projects WHERE github_repo = $1")
        .bind(github_repo)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_approved_project(pool: &PgPool, github_repo: &str) -> Result<Option<Project>> {
    let project = sqlx::query_as::<_, Project>(
        "SELECT id, name, github_repo, forum_channel_id, thread_id, is_approved FROM projects WHERE github_repo = $1 AND is_approved = true"
    )
    .bind(github_repo)
    .fetch_optional(pool)
    .await?;

    Ok(project)
}

pub async fn list_projects(pool: &PgPool) -> Result<Vec<Project>> {
    let projects = sqlx::query_as::<_, Project>(
        "SELECT id, name, github_repo, forum_channel_id, thread_id, is_approved FROM projects ORDER BY is_approved DESC, name ASC"
    )
    .fetch_all(pool)
    .await?;

    Ok(projects)
}

pub async fn update_thread_id(pool: &PgPool, github_repo: &str, thread_id: &str) -> Result<()> {
    sqlx::query("UPDATE projects SET thread_id = $2 WHERE github_repo = $1")
        .bind(github_repo)
        .bind(thread_id)
        .execute(pool)
        .await?;
    Ok(())
}
