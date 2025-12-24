-- Server configuration for ByteHub channels
CREATE TABLE IF NOT EXISTS server_config (
    guild_id TEXT PRIMARY KEY,
    announcements_id TEXT NOT NULL,
    github_forum_id TEXT NOT NULL,
    mod_category_id TEXT,
    project_review_id TEXT,
    approvals_id TEXT,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Store which thread belongs to which project
ALTER TABLE projects ADD COLUMN IF NOT EXISTS thread_id TEXT;
