//! Unit tests for `SignalRegistry`.
//!
//! Tests cover:
//! - Token bucket rate limiting
//! - Broadcast fan-out distribution
//! - Clone isolation (independent rate limiters)
//! - Type bridging (`ObservationSignal` → `ExternalSignal`)

use xiuxian_zhenfa::{
    BroadcastResult, ExternalSignal, ObservationSignalInput, SignalRegistry, ZhenfaSignalType,
};

/// Helper to create a test signal.
fn test_signal() -> ExternalSignal {
    ExternalSignal {
        source: "sentinel".to_string(),
        signal_type: ZhenfaSignalType::from("semantic_drift"),
        summary: "Test signal".to_string(),
        confidence: 0.9,
        affected_docs: vec!["docs/api".to_string()],
        auto_fix_available: true,
        timestamp: "2024-01-01T00:00:00Z".to_string(),
    }
}

#[test]
fn signal_registry_new_has_full_token_bucket() {
    let registry = SignalRegistry::new();
    assert_eq!(registry.subscriber_count(), 0);
    // Should start with full bucket
    assert!(registry.available_tokens() > 0);
}

#[test]
fn signal_registry_available_tokens_decreases_on_broadcast() {
    let registry = SignalRegistry::with_rate_limit(5, 1);

    assert_eq!(registry.available_tokens(), 5);

    registry.broadcast(&test_signal());
    assert_eq!(registry.available_tokens(), 4);

    registry.broadcast(&test_signal());
    assert_eq!(registry.available_tokens(), 3);
}

#[test]
fn signal_registry_rate_limits_when_bucket_empty() {
    let registry = SignalRegistry::with_rate_limit(2, 1);

    // First two broadcasts should consume tokens (returns NoSubscribers since no subscribers)
    assert!(matches!(
        registry.broadcast(&test_signal()),
        BroadcastResult::NoSubscribers
    ));
    assert!(matches!(
        registry.broadcast(&test_signal()),
        BroadcastResult::NoSubscribers
    ));

    // Third should be rate limited
    let result = registry.broadcast(&test_signal());
    assert!(matches!(result, BroadcastResult::RateLimited { .. }));
}

#[test]
fn signal_registry_broadcast_unlimited_bypasses_rate_limit() {
    let registry = SignalRegistry::with_rate_limit(1, 1);

    // Exhaust the rate limit
    registry.broadcast(&test_signal());

    // Should be rate limited now
    assert!(matches!(
        registry.broadcast(&test_signal()),
        BroadcastResult::RateLimited { .. }
    ));

    // But broadcast_unlimited should work (returns 0 since no subscribers)
    let count = registry.broadcast_unlimited(test_signal());
    assert_eq!(count, 0);
}

#[test]
fn signal_registry_no_subscribers_returns_no_subscribers() {
    let registry = SignalRegistry::new();

    let result = registry.broadcast(&test_signal());
    assert!(matches!(result, BroadcastResult::NoSubscribers));
}

#[test]
fn signal_registry_clone_has_independent_rate_limiter() {
    let registry1 = SignalRegistry::with_rate_limit(2, 1);
    let registry2 = registry1.clone();

    // Exhaust registry1's rate limit
    registry1.broadcast(&test_signal());
    registry1.broadcast(&test_signal());

    // registry1 should be rate limited
    assert!(matches!(
        registry1.broadcast(&test_signal()),
        BroadcastResult::RateLimited { .. }
    ));

    // registry2 should still have tokens (independent limiter)
    assert!(matches!(
        registry2.broadcast(&test_signal()),
        BroadcastResult::NoSubscribers // No subscribers, but not rate limited
    ));
}

#[test]
fn convert_observation_signal_creates_valid_external_signal() {
    let external = SignalRegistry::convert_observation_signal(ObservationSignalInput {
        source: "sentinel".to_string(),
        signal_type: ZhenfaSignalType::from("stale"),
        summary: "Observation may be stale".to_string(),
        confidence: 0.75,
        affected_docs: vec!["docs/guide".to_string(), "docs/api".to_string()],
        auto_fix_available: false,
    });

    assert_eq!(external.source, "sentinel");
    assert_eq!(external.signal_type, "stale");
    assert!((external.confidence - 0.75).abs() < 0.001);
    assert_eq!(external.affected_docs.len(), 2);
    assert!(!external.auto_fix_available);
}

#[tokio::test]
async fn signal_registry_fan_out_to_multiple_subscribers() {
    let registry = SignalRegistry::new();

    // Create multiple subscribers (simulating multiple pipelines)
    let mut rx1 = registry.subscribe();
    let mut rx2 = registry.subscribe();
    let mut rx3 = registry.subscribe();

    // Give spawned tasks time to subscribe
    tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;

    assert_eq!(registry.subscriber_count(), 3);

    // Broadcast a signal
    let result = registry.broadcast(&test_signal());
    assert!(matches!(
        result,
        BroadcastResult::Delivered {
            subscriber_count: 3
        }
    ));

    // All three subscribers should receive the same signal
    for (i, rx) in [&mut rx1, &mut rx2, &mut rx3].iter_mut().enumerate() {
        let received = tokio::time::timeout(tokio::time::Duration::from_millis(100), rx.recv())
            .await
            .unwrap_or_else(|_| panic!("Subscriber {} should receive signal", i + 1));
        let Some(received) = received else {
            panic!("Channel should not be closed");
        };

        assert_eq!(received.signal_type, "semantic_drift");
        assert_eq!(received.source, "sentinel");
    }
}

#[tokio::test]
async fn signal_registry_subscribe_increases_subscriber_count() {
    let registry = SignalRegistry::new();
    assert_eq!(registry.subscriber_count(), 0);

    let _rx = registry.subscribe();

    // Give the spawned task time to start and subscribe
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Subscriber count should now be at least 1
    assert!(registry.subscriber_count() >= 1);
}

#[test]
fn broadcast_result_debug_impl() {
    let delivered = BroadcastResult::Delivered {
        subscriber_count: 3,
    };
    let rate_limited = BroadcastResult::RateLimited {
        tokens_remaining: 5,
    };
    let no_subscribers = BroadcastResult::NoSubscribers;

    // Verify Debug trait is implemented
    assert!(format!("{delivered:?}").contains("Delivered"));
    assert!(format!("{rate_limited:?}").contains("RateLimited"));
    assert!(format!("{no_subscribers:?}").contains("NoSubscribers"));
}

#[test]
fn signal_registry_debug_impl() {
    let registry = SignalRegistry::new();
    let debug_str = format!("{registry:?}");

    // Debug should include id and subscriber_count
    assert!(debug_str.contains("SignalRegistry"));
    assert!(debug_str.contains("id"));
    assert!(debug_str.contains("subscriber_count"));
}

#[test]
fn signal_registry_with_capacity_creates_registry() {
    let registry = SignalRegistry::with_capacity(512);
    assert_eq!(registry.subscriber_count(), 0);
    assert!(registry.available_tokens() > 0);
}
