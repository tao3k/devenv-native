use crate::{
    NotificationPayload, NotificationService, WebhookConfig, ZhenfaSignal, ZhenfaSignalType,
};

#[test]
fn notification_service_new() {
    let config = WebhookConfig::default();
    let service = NotificationService::new(config);
    assert!(!service.id().is_empty());
}

#[test]
fn signal_to_payload_semantic_drift() {
    let signal = ZhenfaSignal::SemanticDrift {
        source_path: "src/lib.rs".to_string(),
        file_stem: "lib".to_string(),
        affected_count: 3,
        confidence: "high".to_string(),
        summary: "Code changed".to_string(),
    };

    let payload = NotificationService::signal_to_payload(&signal);
    assert_eq!(payload.signal_type, "semantic_drift");
    assert_eq!(payload.confidence, "high");
    assert!(payload.auto_fix_available);
}

#[test]
fn signal_to_payload_reward() {
    let signal = ZhenfaSignal::Reward {
        episode_id: "ep-123".to_string(),
        value: 0.95,
        source: "sentinel".to_string(),
    };

    let payload = NotificationService::signal_to_payload(&signal);
    assert_eq!(payload.signal_type, "reward");
    assert!(payload.summary.contains("0.95"));
}

#[test]
fn notification_payload_serialization() {
    let payload = NotificationPayload {
        signal_type: ZhenfaSignalType::from("semantic_drift"),
        source: "src/lib.rs".to_string(),
        summary: "Code changed".to_string(),
        confidence: "high".to_string(),
        affected_docs: vec!["docs/api".to_string()],
        timestamp: "2024-01-01T00:00:00Z".to_string(),
        auto_fix_available: true,
        fix_approval_url: Some("https://example.com/fix/123".to_string()),
    };

    let json = match serde_json::to_string(&payload) {
        Ok(json) => json,
        Err(err) => panic!("payload serialization should succeed: {err}"),
    };
    assert!(json.contains("semantic_drift"));
    assert!(json.contains("fix_approval_url"));
}

#[tokio::test]
async fn notification_service_skips_empty_url() {
    let service = NotificationService::new(WebhookConfig::default());
    let payload = NotificationPayload {
        signal_type: ZhenfaSignalType::from("test"),
        source: "test".to_string(),
        summary: "Test".to_string(),
        confidence: "high".to_string(),
        affected_docs: vec![],
        timestamp: "t".to_string(),
        auto_fix_available: false,
        fix_approval_url: None,
    };

    let result = service.notify(&payload).await;
    assert!(result.is_ok());
}
