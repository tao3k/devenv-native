use super::payload::DiffPreview;
use super::rate_limiter::RateLimiter;
use super::{
    AffectedDocInfo, ForwardNotification, ForwardNotifier, ForwarderConfig, SuggestedAction,
};
use crate::zhenfa_router::native::sentinel::{DriftConfidence, SemanticDriftSignal};

#[test]
fn forwarder_config_default() {
    let config = ForwarderConfig::default();
    assert_eq!(config.min_confidence, DriftConfidence::Medium);
    assert!(!config.webhook_enabled);
    assert!(!config.auto_fix_enabled);
}

#[test]
fn forward_notification_serialization() {
    let notification = ForwardNotification {
        id: "notif-123".to_string(),
        source_path: "src/lib.rs".to_string(),
        timestamp: "2024-01-01T00:00:00Z".to_string(),
        affected_docs: vec![AffectedDocInfo {
            doc_id: "docs/api".to_string(),
            pattern: "fn process_data($$$)".to_string(),
            language: "rust".to_string(),
            line_number: Some(42),
            owner: Some("alice".to_string()),
        }],
        confidence: "high".to_string(),
        summary: "Source change may affect docs/api".to_string(),
        suggested_action: SuggestedAction::UpdatePattern,
        auto_fix_available: true,
        diff_preview: Some(DiffPreview {
            lines_added: 5,
            lines_removed: 2,
            unified_diff: "--- a/src/lib.rs\n+++ b/src/lib.rs\n".to_string(),
            symbols_added: vec!["process_records".to_string()],
            symbols_removed: vec!["process_data".to_string()],
            context_lines: 3,
        }),
    };

    let Ok(json) = serde_json::to_string(&notification) else {
        panic!("forward notification should serialize");
    };
    assert!(json.contains("process_data"));
    assert!(json.contains("auto_fix_available"));
    assert!(json.contains("diff_preview"));
    assert!(json.contains("process_records"));
}

#[tokio::test]
async fn rate_limiter_allows_under_limit() {
    let mut limiter = RateLimiter::default();

    for _i in 0..5 {
        assert!(limiter.check_and_increment("doc1", 5));
    }
    assert!(!limiter.check_and_increment("doc1", 5));
}

#[tokio::test]
async fn rate_limiter_different_docs() {
    let mut limiter = RateLimiter::default();

    assert!(limiter.check_and_increment("doc1", 2));
    assert!(limiter.check_and_increment("doc1", 2));
    assert!(!limiter.check_and_increment("doc1", 2));

    // Different doc should still work
    assert!(limiter.check_and_increment("doc2", 2));
}

#[tokio::test]
async fn forward_notifier_low_confidence_skipped() {
    let config = ForwarderConfig {
        min_confidence: DriftConfidence::Medium,
        ..Default::default()
    };
    let notifier = ForwardNotifier::new(config);

    let mut drift = SemanticDriftSignal::new("src/lib.rs", "lib");
    drift.update_confidence(DriftConfidence::Low);

    let result = notifier.process_drift(&drift).await;
    assert!(!result);
}

#[tokio::test]
async fn forward_notifier_high_confidence_processed() {
    let config = ForwarderConfig {
        min_confidence: DriftConfidence::Medium,
        rate_limit_per_hour: 10,
        ..Default::default()
    };
    let notifier = ForwardNotifier::new(config);

    let mut drift = SemanticDriftSignal::new("src/lib.rs", "lib");
    drift.update_confidence(DriftConfidence::High);

    // Without a sender attached, returns false
    let result = notifier.process_drift(&drift).await;
    assert!(!result);
}

#[test]
fn suggested_action_ordering() {
    assert_eq!(SuggestedAction::AutoFix, SuggestedAction::AutoFix);
    assert_ne!(SuggestedAction::Review, SuggestedAction::AutoFix);
}

#[tokio::test]
async fn can_auto_fix_check() {
    let config = ForwarderConfig {
        auto_fix_enabled: true,
        auto_fix_min_confidence: DriftConfidence::High,
        ..Default::default()
    };
    let notifier = ForwardNotifier::new(config);

    let mut drift_high = SemanticDriftSignal::new("src/lib.rs", "lib");
    drift_high.update_confidence(DriftConfidence::High);
    assert!(notifier.can_auto_fix(&drift_high));

    let mut drift_low = SemanticDriftSignal::new("src/lib.rs", "lib");
    drift_low.update_confidence(DriftConfidence::Low);
    assert!(!notifier.can_auto_fix(&drift_low));
}
