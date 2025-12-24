use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{Error, Result};

#[derive(Debug, sqlx::FromRow)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub github_repo: String,
    pub forum_channel_id: String,
    pub is_approved: bool,
}

pub async fn submit_project(
    pool: &PgPool,
    github_repo: &str,
    forum_channel_id: &str,
) -> Result<Uuid> {
    let name = github_repo.split('/').last().unwrap_or(github_repo);
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO projects (name, github_repo, forum_channel_id)
        VALUES ($1, $2, $3)
        ON CONFLICT (github_repo) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(name)
    .bind(github_repo)
    .bind(forum_channel_id)
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

pub async fn deny_project(pool: &PgPool, github_repo: &str) -> Result<()> {
    sqlx::query("DELETE FROM projects WHERE github_repo = $1")
        .bind(github_repo)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_approved_project(pool: &PgPool, github_repo: &str) -> Result<Option<Project>> {
    let project = sqlx::query_as::<_, Project>(
        "SELECT id, name, github_repo, forum_channel_id, is_approved FROM projects WHERE github_repo = $1 AND is_approved = true"
    )
    .bind(github_repo)
    .fetch_optional(pool)
    .await?;

    Ok(project)
}
