//! Unit tests for Gateway Notification Service.
//!
//! Tests cover:
//! - `NotificationService` creation and configuration
//! - Webhook notification behavior
//! - Payload serialization
//! - Error handling

use xiuxian_zhenfa::{
    NotificationError, NotificationPayload, NotificationService, WebhookConfig, ZhenfaSignalType,
};

/// Helper to create a test webhook config.
fn test_config() -> WebhookConfig {
    WebhookConfig {
        url: String::new(),
        secret: None,
        timeout_secs: 5,
        retry_on_failure: true,
    }
}

/// Helper to create a test payload.
fn test_payload() -> NotificationPayload {
    NotificationPayload {
        signal_type: ZhenfaSignalType::from("semantic_drift"),
        source: "src/lib.rs".to_string(),
        summary: "Code changed".to_string(),
        confidence: "high".to_string(),
        affected_docs: vec!["docs/api".to_string()],
        timestamp: "2024-01-01T00:00:00Z".to_string(),
        auto_fix_available: true,
        fix_approval_url: None,
    }
}

#[test]
fn notification_service_new_with_config() {
    let config = test_config();
    let service = NotificationService::new(config);
    assert!(!service.id().is_empty());
    assert!(service.id().starts_with("notif-"));
}

#[test]
fn notification_payload_serialization() {
    let payload = NotificationPayload {
        signal_type: ZhenfaSignalType::from("semantic_drift"),
        source: "src/lib.rs".to_string(),
        summary: "Code changed".to_string(),
        confidence: "high".to_string(),
        affected_docs: vec!["docs/api".to_string(), "docs/guide".to_string()],
        timestamp: "2024-01-01T00:00:00Z".to_string(),
        auto_fix_available: true,
        fix_approval_url: Some("https://example.com/approve/123".to_string()),
    };

    let json = match serde_json::to_string(&payload) {
        Ok(json) => json,
        Err(err) => panic!("payload serialization should succeed: {err}"),
    };
    assert!(json.contains("semantic_drift"));
    assert!(json.contains("docs/api"));
    assert!(json.contains("fix_approval_url"));
    assert!(json.contains("https://example.com/approve/123"));

    // Deserialize back
    let parsed: NotificationPayload = match serde_json::from_str(&json) {
        Ok(parsed) => parsed,
        Err(err) => panic!("payload deserialization should succeed: {err}"),
    };
    assert_eq!(parsed.signal_type, "semantic_drift");
    assert_eq!(parsed.affected_docs.len(), 2);
}

#[test]
fn notification_payload_with_fix_approval_url() {
    let payload = NotificationPayload {
        signal_type: ZhenfaSignalType::from("semantic_drift"),
        source: "src/lib.rs".to_string(),
        summary: "Fix available".to_string(),
        confidence: "high".to_string(),
        affected_docs: vec![],
        timestamp: "2024-01-01T00:00:00Z".to_string(),
        auto_fix_available: true,
        fix_approval_url: Some("https://example.com/approve/fix-123".to_string()),
    };

    assert!(payload.fix_approval_url.is_some());
    let Some(url) = payload.fix_approval_url else {
        panic!("fix approval URL should be present");
    };
    assert!(url.contains("approve"));
}

#[tokio::test]
async fn notification_service_skips_empty_url() {
    let service = NotificationService::new(test_config());
    let payload = test_payload();

    // Empty URL should succeed without error (early return)
    let result = service.notify(&payload).await;
    assert!(result.is_ok());
}

#[test]
fn notification_error_display() {
    let err = NotificationError::HttpError {
        status: 404,
        url: "https://example.com/webhook".to_string(),
    };
    assert!(err.to_string().contains("404"));
    assert!(err.to_string().contains("example.com"));

    let err = NotificationError::NetworkError("connection refused".to_string());
    assert!(err.to_string().contains("connection refused"));

    let err = NotificationError::MaxRetriesExceeded {
        attempts: 3,
        url: "https://example.com/webhook".to_string(),
    };
    assert!(err.to_string().contains('3'));
}

#[test]
fn webhook_config_default() {
    let config = WebhookConfig::default();
    assert!(config.url.is_empty());
    assert!(config.secret.is_none());
    assert_eq!(config.timeout_secs, 10);
    assert!(config.retry_on_failure);
}

#[test]
fn webhook_config_custom() {
    let config = WebhookConfig {
        url: "https://hooks.slack.com/services/xxx".to_string(),
        secret: Some("my-secret".to_string()),
        timeout_secs: 30,
        retry_on_failure: false,
    };

    assert_eq!(config.url, "https://hooks.slack.com/services/xxx");
    assert_eq!(config.secret, Some("my-secret".to_string()));
    assert_eq!(config.timeout_secs, 30);
    assert!(!config.retry_on_failure);
}

#[test]
fn notification_service_with_shared_client() {
    let client = reqwest::Client::new();
    let config = test_config();
    let service = NotificationService::with_client(config, client);

    assert!(!service.id().is_empty());
}

#[test]
fn notification_payload_deserialization() {
    let json = r#"{
        "signal_type": "reward",
        "source": "sentinel",
        "summary": "Episode completed",
        "confidence": "high",
        "affected_docs": [],
        "timestamp": "2024-01-01T00:00:00Z",
        "auto_fix_available": false,
        "fix_approval_url": null
    }"#;

    let payload: NotificationPayload = match serde_json::from_str(json) {
        Ok(payload) => payload,
        Err(err) => panic!("payload deserialization should succeed: {err}"),
    };
    assert_eq!(payload.signal_type, "reward");
    assert_eq!(payload.source, "sentinel");
    assert!(!payload.auto_fix_available);
    assert!(payload.fix_approval_url.is_none());
}
