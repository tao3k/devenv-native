use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};

use super::ingest;
use crate::channels::telegram::runtime::webhook::auth::validate_secret_token;
use crate::channels::telegram::runtime::webhook::dedup::is_duplicate_update;
use crate::channels::telegram::runtime::webhook::state::TelegramWebhookState;

pub(in crate::channels::telegram::runtime::webhook) async fn telegram_webhook_handler(
    State(state): State<TelegramWebhookState>,
    headers: HeaderMap,
    Json(update): Json<serde_json::Value>,
) -> Result<StatusCode, (StatusCode, String)> {
    let update_id = update.get("update_id").and_then(serde_json::Value::as_i64);
    tracing::info!(
        update_id = ?update_id,
        "Webhook received Telegram update"
    );

    validate_secret_token(&headers, state.secret_token.as_deref())?;

    if let Some(update_id) = update_id
        && is_duplicate_update(&state, update_id).await
    {
        tracing::debug!(update_id, "Skipping duplicate update");
        return Ok(StatusCode::OK);
    }

    ingest::forward_update_to_agent(&state, &update).await?;
    Ok(StatusCode::OK)
}
