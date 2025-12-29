import { query, mutation } from "./_generated/server";
import { v } from "convex/values";

// Result type for mutations
type MutationResult =
    | { success: true; id: string }
    | { success: false; error: string };

// Submit a new project for approval
export const submit = mutation({
    args: { github_repo: v.string() },
    handler: async (ctx, args): Promise<MutationResult> => {
        const github_repo = args.github_repo.toLowerCase();
        const name = github_repo.split("/").pop() || github_repo;

        // Check if project already exists
        const existing = await ctx.db
            .query("projects")
            .withIndex("by_github_repo", (q) => q.eq("github_repo", github_repo))
            .first();

        if (existing) {
            return { success: false, error: "Project already exists" };
        }

        const id = await ctx.db.insert("projects", {
            name,
            github_repo,
            forum_channel_id: "",
            guild_id: "",
            is_approved: false,
        });

        return { success: true, id: id };
    },
});

// Approve a project with forum channel assignment
export const approveWithForum = mutation({
    args: {
        github_repo: v.string(),
        forum_channel_id: v.string(),
        guild_id: v.string(),
    },
    handler: async (ctx, args): Promise<MutationResult> => {
        const project = await ctx.db
            .query("projects")
            .withIndex("by_github_repo", (q) =>
                q.eq("github_repo", args.github_repo.toLowerCase())
            )
            .first();

        if (!project) {
            return { success: false, error: "Project not found" };
        }

        await ctx.db.patch(project._id, {
            is_approved: true,
            forum_channel_id: args.forum_channel_id,
            guild_id: args.guild_id,
        });

        // Create default rules for this project
        const defaultRules = [
            {
                conditions: { event_type: "workflow_run.completed" },
                actions: { post_forum: true, post_announce: false },
            },
            {
                conditions: { event_type: "release.published" },
                actions: { post_forum: true, post_announce: true },
            },
            {
                conditions: { event_type: "pull_request.closed", merged: true },
                actions: { post_forum: true, post_announce: false },
            },
            {
                conditions: { event_type: "issues.opened" },
                actions: { post_forum: true, post_announce: false },
            },
        ];

        for (let i = 0; i < defaultRules.length; i++) {
            await ctx.db.insert("rules", {
                project_id: project._id,
                priority: i,
                conditions: defaultRules[i].conditions,
                actions: defaultRules[i].actions,
            });
        }

        return { success: true, id: project._id };
    },
});

// Simple approve (without forum)
export const approve = mutation({
    args: { github_repo: v.string() },
    handler: async (ctx, args): Promise<MutationResult> => {
        const project = await ctx.db
            .query("projects")
            .withIndex("by_github_repo", (q) =>
                q.eq("github_repo", args.github_repo.toLowerCase())
            )
            .first();

        if (!project) {
            return { success: false, error: "Project not found" };
        }

        await ctx.db.patch(project._id, { is_approved: true });
        return { success: true, id: project._id };
    },
});

// Deny/delete a project
export const deny = mutation({
    args: { github_repo: v.string() },
    handler: async (ctx, args): Promise<{ success: true } | { success: false; error: string }> => {
        const project = await ctx.db
            .query("projects")
            .withIndex("by_github_repo", (q) =>
                q.eq("github_repo", args.github_repo.toLowerCase())
            )
            .first();

        if (!project) {
            return { success: false, error: "Project not found" };
        }

        // Delete associated rules first
        const rules = await ctx.db
            .query("rules")
            .withIndex("by_project", (q) => q.eq("project_id", project._id))
            .collect();

        for (const rule of rules) {
            await ctx.db.delete(rule._id);
        }

        await ctx.db.delete(project._id);
        return { success: true };
    },
});

// Get a project by repo (approved only)
export const getApproved = query({
    args: { github_repo: v.string() },
    handler: async (ctx, args) => {
        const project = await ctx.db
            .query("projects")
            .withIndex("by_github_repo", (q) =>
                q.eq("github_repo", args.github_repo.toLowerCase())
            )
            .first();

        if (project && project.is_approved) {
            return project;
        }
        return null;
    },
});

// Get a project by repo (any status)
export const get = query({
    args: { github_repo: v.string() },
    handler: async (ctx, args) => {
        return await ctx.db
            .query("projects")
            .withIndex("by_github_repo", (q) =>
                q.eq("github_repo", args.github_repo.toLowerCase())
            )
            .first();
    },
});

// List projects by guild
export const listByGuild = query({
    args: { guild_id: v.string() },
    handler: async (ctx, args) => {
        return await ctx.db
            .query("projects")
            .withIndex("by_guild", (q) => q.eq("guild_id", args.guild_id))
            .collect();
    },
});

// Update forum channel ID
export const updateForumId = mutation({
    args: { github_repo: v.string(), forum_id: v.string() },
    handler: async (ctx, args): Promise<{ success: boolean }> => {
        const project = await ctx.db
            .query("projects")
            .withIndex("by_github_repo", (q) =>
                q.eq("github_repo", args.github_repo.toLowerCase())
            )
            .first();

        if (project) {
            await ctx.db.patch(project._id, { forum_channel_id: args.forum_id });
            return { success: true };
        }
        return { success: false };
    },
});

// Update thread ID
export const updateThreadId = mutation({
    args: { github_repo: v.string(), thread_id: v.string() },
    handler: async (ctx, args): Promise<{ success: boolean }> => {
        const project = await ctx.db
            .query("projects")
            .withIndex("by_github_repo", (q) =>
                q.eq("github_repo", args.github_repo.toLowerCase())
            )
            .first();

        if (project) {
            await ctx.db.patch(project._id, { thread_id: args.thread_id });
            return { success: true };
        }
        return { success: false };
    },
});
