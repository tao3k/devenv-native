use tokio::sync::mpsc;
use xiuxian_zhenfa::ZhenfaSignal;

use axum::extract::State;

use crate::bin_support::wendao::execute::gateway::status::{notify_status, stats};

use super::support::{app_state, app_state_with_webhook_url};

#[tokio::test]
async fn test_stats_endpoint_no_index() {
    let state = app_state(None);
    let result = stats(State(state)).await;
    assert_eq!(result.0["error"], "no index loaded");
}

#[tokio::test]
async fn test_notify_status_endpoint() {
    let (tx, _rx) = mpsc::unbounded_channel();
    let state = app_state_with_webhook_url(Some(tx), Some("http://127.0.0.1:9999/hooks".into()));
    let expected_bootstrap_background_indexing =
        state.studio.bootstrap_background_indexing_enabled();
    let expected_bootstrap_background_indexing_mode =
        state.studio.bootstrap_background_indexing_mode();
    let result = notify_status(State(state)).await;
    assert_eq!(result.0["notification_worker"], "active");
    assert_eq!(
        result.0["studio_bootstrap_background_indexing_enabled"],
        serde_json::json!(expected_bootstrap_background_indexing)
    );
    assert_eq!(
        result.0["studio_bootstrap_background_indexing_mode"],
        expected_bootstrap_background_indexing_mode
    );
    assert_eq!(
        result.0["studio_bootstrap_background_indexing_deferred_activation_observed"],
        serde_json::json!(false)
    );
    assert!(result.0["studio_bootstrap_background_indexing_deferred_activation_at"].is_null());
    assert!(result.0["studio_bootstrap_background_indexing_deferred_activation_source"].is_null());
    assert_eq!(result.0["webhook_configured"], serde_json::json!(true));
    assert_eq!(
        result.0["webhook_url"],
        serde_json::json!("http://127.0.0.1:9999/hooks")
    );
}

#[tokio::test]
async fn test_notify_status_no_channel() {
    let state = app_state(None);
    let expected_bootstrap_background_indexing =
        state.studio.bootstrap_background_indexing_enabled();
    let expected_bootstrap_background_indexing_mode =
        state.studio.bootstrap_background_indexing_mode();
    let result = notify_status(State(state)).await;
    assert_eq!(result.0["notification_worker"], "inactive");
    assert_eq!(
        result.0["studio_bootstrap_background_indexing_enabled"],
        serde_json::json!(expected_bootstrap_background_indexing)
    );
    assert_eq!(
        result.0["studio_bootstrap_background_indexing_mode"],
        expected_bootstrap_background_indexing_mode
    );
    assert_eq!(
        result.0["studio_bootstrap_background_indexing_deferred_activation_observed"],
        serde_json::json!(false)
    );
    assert!(result.0["studio_bootstrap_background_indexing_deferred_activation_at"].is_null());
    assert!(result.0["studio_bootstrap_background_indexing_deferred_activation_source"].is_null());
    assert_eq!(result.0["webhook_configured"], serde_json::json!(false));
    assert!(result.0["webhook_url"].is_null());
}

#[tokio::test]
async fn test_notification_channel() {
    let (tx, mut rx) = mpsc::unbounded_channel::<ZhenfaSignal>();
    let signal = ZhenfaSignal::SemanticDrift {
        source_path: "test.rs".to_string(),
        file_stem: "test".to_string(),
        affected_count: 1,
        confidence: "high".to_string(),
        summary: "Test drift".to_string(),
    };
    assert!(tx.send(signal).is_ok());

    let Some(received) = rx.recv().await else {
        panic!("notification channel should receive the test signal");
    };
    assert!(matches!(received, ZhenfaSignal::SemanticDrift { .. }));
}
