// Whitelist module - future work
// The schema has the whitelist table defined but functionality is not yet implemented

use crate::error::Result;
use crate::storage::convex::ConvexDb;

#[allow(dead_code)]
pub async fn add_user(_db: &ConvexDb, _github_username: &str) -> Result<()> {
    // TODO: Implement when whitelist feature is needed
    Ok(())
}

#[allow(dead_code)]
pub async fn is_whitelisted(_db: &ConvexDb, _github_username: &str) -> Result<bool> {
    // TODO: Implement when whitelist feature is needed
    // For now, return false (no one is whitelisted)
    Ok(false)
}
