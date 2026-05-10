//! Observability and telemetry helpers for the agent lifecycle.

use crate::Agent;
use crate::contracts::OmegaDecision;
use crate::observability::SessionEvent;
use std::time::{SystemTime, UNIX_EPOCH};
use xiuxian_qianhuan::InjectionSnapshot;

impl Agent {
    /// Returns a new monotonic turn ID based on current system time.
    pub(crate) fn next_runtime_turn_id() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|duration| u64::try_from(duration.as_millis()).ok())
            .unwrap_or_default()
    }

    /// Records an injection snapshot for telemetry and auditing.
    pub(crate) fn record_injection_snapshot(session_id: &str, snapshot: &InjectionSnapshot) {
        tracing::debug!(
            event = SessionEvent::InjectionSnapshotCreated.as_str(),
            session_id,
            snapshot_id = %snapshot.snapshot_id,
            turn_id = snapshot.turn_id.value(),
            policy_max_blocks = snapshot.policy.max_blocks,
            policy_max_chars = snapshot.policy.max_chars,
            role_mix_profile = snapshot
                .role_mix
                .as_ref()
                .map(|profile| profile.profile_id.as_str()),
            block_count = snapshot.blocks.len(),
            total_chars = snapshot.total_chars,
            dropped_blocks = snapshot.dropped_block_ids.len(),
            truncated_blocks = snapshot.truncated_block_ids.len(),
            dropped_block_ids = ?snapshot.dropped_block_ids,
            truncated_block_ids = ?snapshot.truncated_block_ids,
            "injection snapshot created"
        );
        for block_id in &snapshot.dropped_block_ids {
            tracing::debug!(
                event = SessionEvent::InjectionBlockDropped.as_str(),
                session_id,
                snapshot_id = %snapshot.snapshot_id,
                turn_id = snapshot.turn_id.value(),
                block_id = %block_id,
                "injection block dropped by policy"
            );
        }
        for block_id in &snapshot.truncated_block_ids {
            tracing::debug!(
                event = SessionEvent::InjectionBlockTruncated.as_str(),
                session_id,
                snapshot_id = %snapshot.snapshot_id,
                turn_id = snapshot.turn_id.value(),
                block_id = %block_id,
                "injection block truncated by policy"
            );
        }
    }

    /// Records an omega routing decision.
    pub(crate) fn record_omega_decision(
        session_id: &str,
        decision: &OmegaDecision,
        reason: Option<&str>,
        meta: Option<&serde_json::Value>,
    ) {
        tracing::debug!(
            event = SessionEvent::RouteDecisionSelected.as_str(),
            session_id,
            route = decision.route.as_str(),
            confidence = decision.confidence,
            risk_level = decision.risk_level.as_str(),
            fallback_policy = decision.fallback_policy.as_str(),
            tool_trust_class = decision.tool_trust_class.as_str(),
            policy_id = decision.policy_id.as_deref().unwrap_or(""),
            drift_tolerance = decision.drift_tolerance,
            next_audit_turn = decision.next_audit_turn,
            decision_reason = %decision.reason,
            override_reason = reason.unwrap_or(""),
            meta = ?meta,
            "omega route decision recorded"
        );
    }
}
