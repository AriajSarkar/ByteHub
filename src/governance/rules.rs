use convex::Value as ConvexValue;
use maplit::btreemap;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::github::events::ParsedEvent;
use crate::storage::convex::ConvexDb;

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

pub struct RuleMatch {
    pub actions: RuleActions,
    pub rule_id: String,
}

pub async fn evaluate_rules(
    db: &ConvexDb,
    project_id: &str,
    event: &ParsedEvent,
) -> Result<Option<RuleMatch>> {
    let event_key = event.event_key();
    let is_merged = event.is_merged();

    let result = db
        .query(
            "rules:evaluateForProject",
            btreemap! {
                "project_id".into() => ConvexValue::String(project_id.to_string()),
                "event_key".into() => match &event_key {
                    Some(k) => ConvexValue::String(k.clone()),
                    None => ConvexValue::Null,
                },
                "is_merged".into() => ConvexValue::Boolean(is_merged),
            },
        )
        .await?;

    if result.is_null() {
        return Ok(None);
    }

    let obj = result
        .as_object()
        .ok_or_else(|| Error::InvalidPayload("Expected object from evaluate".into()))?;

    let rule_id = obj
        .get("rule_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::InvalidPayload("Missing rule_id".into()))?
        .to_string();

    let actions_value = obj
        .get("actions")
        .ok_or_else(|| Error::InvalidPayload("Missing actions".into()))?;

    let actions: RuleActions = serde_json::from_value(actions_value.clone())
        .map_err(|e| Error::InvalidPayload(format!("Failed to parse actions: {}", e)))?;

    Ok(Some(RuleMatch { actions, rule_id }))
}
