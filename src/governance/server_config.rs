use sqlx::PgPool;

use crate::error::Result;

#[derive(Debug, sqlx::FromRow)]
pub struct ServerConfig {
    pub guild_id: String,
    pub announcements_id: String,
    pub github_forum_id: String,
    pub mod_category_id: Option<String>,
    pub project_review_id: Option<String>,
    pub approvals_id: Option<String>,
}

/// Get server config by guild ID
pub async fn get_config(pool: &PgPool, guild_id: &str) -> Result<Option<ServerConfig>> {
    let config =
        sqlx::query_as::<_, ServerConfig>("SELECT * FROM server_config WHERE guild_id = $1")
            .bind(guild_id)
            .fetch_optional(pool)
            .await?;
    Ok(config)
}

/// Save server config
pub async fn save_config(
    pool: &PgPool,
    guild_id: &str,
    announcements_id: &str,
    github_forum_id: &str,
    mod_category_id: Option<&str>,
    project_review_id: Option<&str>,
    approvals_id: Option<&str>,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO server_config (guild_id, announcements_id, github_forum_id, mod_category_id, project_review_id, approvals_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (guild_id) DO UPDATE SET
            announcements_id = EXCLUDED.announcements_id,
            github_forum_id = EXCLUDED.github_forum_id,
            mod_category_id = EXCLUDED.mod_category_id,
            project_review_id = EXCLUDED.project_review_id,
            approvals_id = EXCLUDED.approvals_id
        "#
    )
    .bind(guild_id)
    .bind(announcements_id)
    .bind(github_forum_id)
    .bind(mod_category_id)
    .bind(project_review_id)
    .bind(approvals_id)
    .execute(pool)
    .await?;
    Ok(())
}
