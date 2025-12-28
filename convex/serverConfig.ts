import { query, mutation } from "./_generated/server";
import { v } from "convex/values";

// Get server config by guild ID
export const get = query({
    args: { guild_id: v.string() },
    handler: async (ctx, args) => {
        return await ctx.db
            .query("server_config")
            .withIndex("by_guild", (q) => q.eq("guild_id", args.guild_id))
            .first();
    },
});

// Save/update server config with idempotency check
// Only writes to database if data has actually changed
export const save = mutation({
    args: {
        guild_id: v.string(),
        announcements_id: v.string(),
        github_forum_id: v.string(),
        mod_category_id: v.optional(v.string()),
        project_review_id: v.optional(v.string()),
        approvals_id: v.optional(v.string()),
    },
    handler: async (ctx, args) => {
        const existing = await ctx.db
            .query("server_config")
            .withIndex("by_guild", (q) => q.eq("guild_id", args.guild_id))
            .first();

        // Build update object
        const updates = {
            announcements_id: args.announcements_id,
            github_forum_id: args.github_forum_id,
            mod_category_id: args.mod_category_id,
            project_review_id: args.project_review_id,
            approvals_id: args.approvals_id,
        };

        if (existing) {
            // IDEMPOTENCY: Only update if data actually changed
            // This prevents write conflicts when the same data is saved multiple times
            const hasChanges =
                existing.announcements_id !== args.announcements_id ||
                existing.github_forum_id !== args.github_forum_id ||
                existing.mod_category_id !== args.mod_category_id ||
                existing.project_review_id !== args.project_review_id ||
                existing.approvals_id !== args.approvals_id;

            if (!hasChanges) {
                // No changes needed, return early without write
                return existing._id;
            }

            await ctx.db.patch(existing._id, updates);
            return existing._id;
        }

        return await ctx.db.insert("server_config", {
            guild_id: args.guild_id,
            ...updates,
        });
    },
});
