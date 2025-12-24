use sqlx::PgPool;

use crate::error::Result;

pub async fn add_user(pool: &PgPool, github_username: &str) -> Result<()> {
    sqlx::query("INSERT INTO whitelist (github_username) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(github_username)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn is_whitelisted(pool: &PgPool, github_username: &str) -> Result<bool> {
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM whitelist WHERE github_username = $1)",
    )
    .bind(github_username)
    .fetch_one(pool)
    .await?;
    Ok(exists)
}
