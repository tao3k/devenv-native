//! Top-level integration tests for `agent::admission`.

use std::collections::HashMap;

use xiuxian_daochang::test_support::{
    DownstreamAdmissionDecision, DownstreamAdmissionMetrics, DownstreamAdmissionPolicy,
    DownstreamAdmissionRejectReason, DownstreamInFlightSnapshot, DownstreamRuntimeSnapshot,
};

#[test]
fn admission_policy_defaults_when_env_unset() {
    let policy = DownstreamAdmissionPolicy::from_lookup(|_| None);
    assert!(policy.enabled);
    assert_eq!(policy.llm_reject_threshold_pct, 95);
    assert_eq!(policy.embedding_reject_threshold_pct, 95);
}

#[test]
fn admission_policy_respects_disable_env() {
    let values = HashMap::from([(
        "XIUXIAN_DAOCHANG_DOWNSTREAM_ADMISSION_ENABLED".to_string(),
        "false".to_string(),
    )]);
    let policy = DownstreamAdmissionPolicy::from_lookup(|name| values.get(name).cloned());
    assert!(!policy.enabled);
}

#[test]
fn admission_policy_invalid_threshold_falls_back_to_default() {
    let values = HashMap::from([(
        "XIUXIAN_DAOCHANG_ADMISSION_LLM_SATURATION_PCT".to_string(),
        "0".to_string(),
    )]);
    let policy = DownstreamAdmissionPolicy::from_lookup(|name| values.get(name).cloned());
    assert_eq!(policy.llm_reject_threshold_pct, 95);
}

#[test]
fn admission_policy_rejects_llm_saturation_first() {
    let policy = DownstreamAdmissionPolicy {
        enabled: true,
        llm_reject_threshold_pct: 90,
        embedding_reject_threshold_pct: 90,
    };
    let decision = policy.evaluate(DownstreamRuntimeSnapshot {
        llm: Some(snapshot(96)),
        embedding: Some(snapshot(96)),
    });
    assert!(!decision.admitted);
    assert_eq!(
        decision.reason,
        Some(DownstreamAdmissionRejectReason::LlmSaturated)
    );
}

#[test]
fn admission_policy_rejects_embedding_saturation() {
    let policy = DownstreamAdmissionPolicy {
        enabled: true,
        llm_reject_threshold_pct: 90,
        embedding_reject_threshold_pct: 90,
    };
    let decision = policy.evaluate(DownstreamRuntimeSnapshot {
        llm: Some(snapshot(40)),
        embedding: Some(snapshot(95)),
    });
    assert!(!decision.admitted);
    assert_eq!(
        decision.reason,
        Some(DownstreamAdmissionRejectReason::EmbeddingSaturated)
    );
}

#[test]
fn admission_policy_allows_when_below_threshold() {
    let policy = DownstreamAdmissionPolicy {
        enabled: true,
        llm_reject_threshold_pct: 90,
        embedding_reject_threshold_pct: 90,
    };
    let decision = policy.evaluate(DownstreamRuntimeSnapshot {
        llm: Some(snapshot(80)),
        embedding: Some(snapshot(50)),
    });
    assert!(decision.admitted);
    assert_eq!(decision.reason, None);
}

#[test]
fn admission_metrics_snapshot_defaults_to_zero() {
    let metrics = DownstreamAdmissionMetrics::default();
    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.total, 0);
    assert_eq!(snapshot.admitted, 0);
    assert_eq!(snapshot.rejected, 0);
    assert_eq!(snapshot.rejected_llm_saturated, 0);
    assert_eq!(snapshot.rejected_embedding_saturated, 0);
    assert_eq!(snapshot.reject_rate_pct, 0);
}

#[test]
fn admission_metrics_tracks_reject_reasons_and_rate() {
    let metrics = DownstreamAdmissionMetrics::default();
    metrics.observe(decision(true, None));
    metrics.observe(decision(
        false,
        Some(DownstreamAdmissionRejectReason::LlmSaturated),
    ));
    metrics.observe(decision(
        false,
        Some(DownstreamAdmissionRejectReason::EmbeddingSaturated),
    ));
    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.total, 3);
    assert_eq!(snapshot.admitted, 1);
    assert_eq!(snapshot.rejected, 2);
    assert_eq!(snapshot.rejected_llm_saturated, 1);
    assert_eq!(snapshot.rejected_embedding_saturated, 1);
    assert_eq!(snapshot.reject_rate_pct, 66);
}

fn snapshot(saturation_pct: u8) -> DownstreamInFlightSnapshot {
    DownstreamInFlightSnapshot {
        max_in_flight: 100,
        available_permits: usize::from(100_u8.saturating_sub(saturation_pct)),
        in_flight: usize::from(saturation_pct),
        saturation_pct,
    }
}

fn decision(
    admitted: bool,
    reason: Option<DownstreamAdmissionRejectReason>,
) -> DownstreamAdmissionDecision {
    DownstreamAdmissionDecision {
        admitted,
        reason,
        snapshot: DownstreamRuntimeSnapshot::default(),
        llm_reject_threshold_pct: 95,
        embedding_reject_threshold_pct: 95,
    }
}
