-- Create projects table
CREATE TABLE IF NOT EXISTS projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    github_repo TEXT UNIQUE NOT NULL,
    forum_channel_id TEXT NOT NULL,
    is_approved BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT now()
);

-- Create whitelist table
CREATE TABLE IF NOT EXISTS whitelist (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    github_username TEXT UNIQUE NOT NULL
);

-- Create moderators table
CREATE TABLE IF NOT EXISTS moderators (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    discord_id TEXT UNIQUE NOT NULL
);

-- Create rules table with composable JSONB conditions and actions
CREATE TABLE IF NOT EXISTS rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID REFERENCES projects(id) ON DELETE CASCADE,
    priority INT DEFAULT 0,
    conditions JSONB NOT NULL,
    actions JSONB NOT NULL
);

-- Index for faster rule lookups
CREATE INDEX IF NOT EXISTS idx_rules_project_id ON rules(project_id);
CREATE INDEX IF NOT EXISTS idx_rules_priority ON rules(priority DESC);
