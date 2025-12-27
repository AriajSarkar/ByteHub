import { defineSchema, defineTable } from "convex/server";
import { v } from "convex/values";

export default defineSchema({
    // GitHub projects tracked by ByteHub
    projects: defineTable({
        name: v.string(),
        github_repo: v.string(),
        forum_channel_id: v.string(),
        thread_id: v.optional(v.string()),
        guild_id: v.string(),
        is_approved: v.boolean(),
    })
        .index("by_github_repo", ["github_repo"])
        .index("by_guild", ["guild_id"]),

    // Whitelisted GitHub usernames (future work)
    whitelist: defineTable({
        github_username: v.string(),
    }).index("by_username", ["github_username"]),

    // Discord moderator IDs
    moderators: defineTable({
        discord_id: v.string(),
    }).index("by_discord_id", ["discord_id"]),

    // Composable rules for event routing
    rules: defineTable({
        project_id: v.id("projects"),
        priority: v.number(),
        conditions: v.any(),
        actions: v.any(),
    }).index("by_project", ["project_id"]),

    // Discord server channel configuration
    server_config: defineTable({
        guild_id: v.string(),
        announcements_id: v.string(),
        github_forum_id: v.string(),
        mod_category_id: v.optional(v.string()),
        project_review_id: v.optional(v.string()),
        approvals_id: v.optional(v.string()),
    }).index("by_guild", ["guild_id"]),
});
