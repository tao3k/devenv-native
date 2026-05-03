//! Gateway status endpoints.

use std::sync::Arc;

use axum::{Json, extract::State};
use serde_json::json;

use crate::bin_support::wendao::execute::gateway::shared::AppState;
use xiuxian_wendao::LinkGraphIndex;

/// Stats endpoint.
pub(crate) async fn stats(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    match &state.index {
        Some(index) => {
            let payload = LinkGraphIndex::stats(index.as_ref());
            Json(
                serde_json::to_value(payload)
                    .unwrap_or_else(|_| json!({"error": "serialization failed"})),
            )
        }
        None => Json(json!({"error": "no index loaded"})),
    }
}

/// Notification service status endpoint.
pub(crate) async fn notify_status(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let has_signal_channel = state.signal_tx.is_some();
    let webhook_url = state.webhook_url.clone();
    let telemetry = state.studio.bootstrap_background_indexing_telemetry();

    Json(json!({
        "notification_worker": if has_signal_channel { "active" } else { "inactive" },
        "studio_bootstrap_background_indexing_enabled": telemetry.enabled(),
        "studio_bootstrap_background_indexing_mode": telemetry.mode(),
        "studio_bootstrap_background_indexing_deferred_activation_observed": telemetry
            .deferred_activation_observed(),
        "studio_bootstrap_background_indexing_deferred_activation_at": telemetry
            .deferred_activation_at(),
        "studio_bootstrap_background_indexing_deferred_activation_source": telemetry
            .deferred_activation_source(),
        "webhook_configured": webhook_url.is_some(),
        "webhook_url": webhook_url.map_or(serde_json::Value::Null, serde_json::Value::String)
    }))
}
