-- Add guild_id to projects table for announcement lookups
ALTER TABLE projects ADD COLUMN IF NOT EXISTS guild_id TEXT DEFAULT '';
