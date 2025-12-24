use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::Result;
use crate::github::events::ParsedEvent;
use crate::governance::whitelist;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConditions {
    pub event_type: Option<String>,
    pub labels: Option<Vec<String>>,
    pub actor_whitelisted: Option<bool>,
    pub merged: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleActions {
    pub post_forum: bool,
    pub post_announce: bool,
    pub template: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Rule {
    pub id: Uuid,
    pub project_id: Uuid,
    pub priority: i32,
    pub conditions: serde_json::Value,
    pub actions: serde_json::Value,
}

pub struct RuleMatch {
    pub actions: RuleActions,
    pub rule_id: Uuid,
}

pub async fn evaluate_rules(
    pool: &PgPool,
    project_id: Uuid,
    event: &ParsedEvent,
) -> Result<Option<RuleMatch>> {
    let rules = sqlx::query_as::<_, Rule>(
        "SELECT id, project_id, priority, conditions, actions FROM rules WHERE project_id = $1 ORDER BY priority DESC"
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    let event_key = event.event_key();
    let event_labels = event.labels();
    let actor = event.actor();
    let is_merged = event.is_merged();

    for rule in rules {
        let conditions: RuleConditions =
            serde_json::from_value(rule.conditions.clone()).unwrap_or(RuleConditions {
                event_type: None,
                labels: None,
                actor_whitelisted: None,
                merged: None,
            });

        if !matches_conditions(
            pool,
            &conditions,
            &event_key,
            &event_labels,
            actor,
            is_merged,
        )
        .await?
        {
            continue;
        }

        let actions: RuleActions =
            serde_json::from_value(rule.actions.clone()).unwrap_or(RuleActions {
                post_forum: false,
                post_announce: false,
                template: None,
            });

        return Ok(Some(RuleMatch {
            actions,
            rule_id: rule.id,
        }));
    }

    Ok(None)
}

async fn matches_conditions(
    pool: &PgPool,
    conditions: &RuleConditions,
    event_key: &Option<String>,
    event_labels: &[String],
    actor: Option<&str>,
    is_merged: bool,
) -> Result<bool> {
    if let Some(ref required_type) = conditions.event_type {
        if event_key.as_ref() != Some(required_type) {
            return Ok(false);
        }
    }

    if let Some(ref required_labels) = conditions.labels {
        for label in required_labels {
            if !event_labels.iter().any(|l| l == label) {
                return Ok(false);
            }
        }
    }

    if let Some(require_whitelisted) = conditions.actor_whitelisted {
        if require_whitelisted {
            if let Some(actor_name) = actor {
                if !whitelist::is_whitelisted(pool, actor_name).await? {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }
    }

    if let Some(require_merged) = conditions.merged {
        if require_merged && !is_merged {
            return Ok(false);
        }
    }

    Ok(true)
}
