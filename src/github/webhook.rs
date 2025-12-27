use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use tracing::{info, warn};

use crate::error::{Error, Result};
use crate::github::{events::ParsedEvent, verify::verify_github_signature};
use crate::router::dispatch::Dispatcher;
use crate::AppState;

pub async fn handle_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse> {
    let signature = headers
        .get("x-hub-signature-256")
        .and_then(|v| v.to_str().ok())
        .ok_or(Error::InvalidSignature)?;

    if !verify_github_signature(&state.config.github_webhook_secret, &body, signature) {
        warn!("invalid github signature");
        return Err(Error::InvalidSignature);
    }

    let event_type = headers
        .get("x-github-event")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| Error::InvalidPayload("missing x-github-event header".into()))?;

    let event = ParsedEvent::from_payload(event_type, &body)
        .map_err(|e| Error::InvalidPayload(e.to_string()))?;

    if matches!(event, ParsedEvent::Unknown) {
        info!(event_type, "ignoring unknown event type");
        return Ok(StatusCode::OK);
    }

    let dispatcher = Dispatcher::new(state.db.clone(), state.discord.clone());
    dispatcher.dispatch(event).await?;

    Ok(StatusCode::OK)
}
