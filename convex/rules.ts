import { query, mutation } from "./_generated/server";
import { v } from "convex/values";

// Get rules for a project
export const getByProject = query({
    args: { project_id: v.id("projects") },
    handler: async (ctx, args) => {
        return await ctx.db
            .query("rules")
            .withIndex("by_project", (q) => q.eq("project_id", args.project_id))
            .order("desc")
            .collect();
    },
});

// Evaluate rules and find matching actions
export const evaluateForProject = query({
    args: {
        project_id: v.id("projects"),
        event_key: v.optional(v.string()),
        is_merged: v.boolean(),
    },
    handler: async (ctx, args) => {
        const rules = await ctx.db
            .query("rules")
            .withIndex("by_project", (q) => q.eq("project_id", args.project_id))
            .order("desc")
            .collect();

        // Sort by priority descending
        rules.sort((a, b) => b.priority - a.priority);

        for (const rule of rules) {
            const conditions = rule.conditions as {
                event_type?: string;
                merged?: boolean;
            };

            // Check event_type condition
            if (conditions.event_type && conditions.event_type !== args.event_key) {
                continue;
            }

            // Check merged condition
            if (conditions.merged !== undefined && conditions.merged !== args.is_merged) {
                continue;
            }

            // All conditions matched
            return {
                rule_id: rule._id,
                actions: rule.actions,
            };
        }

        return null;
    },
});

// Create a rule
export const create = mutation({
    args: {
        project_id: v.id("projects"),
        priority: v.number(),
        conditions: v.any(),
        actions: v.any(),
    },
    handler: async (ctx, args) => {
        return await ctx.db.insert("rules", {
            project_id: args.project_id,
            priority: args.priority,
            conditions: args.conditions,
            actions: args.actions,
        });
    },
});
