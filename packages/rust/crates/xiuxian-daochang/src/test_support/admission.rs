//! Downstream-admission helpers exposed for integration tests.

use crate::agent::admission as internal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Test-facing reason why downstream admission rejected a turn.
pub enum DownstreamAdmissionRejectReason {
    LlmSaturated,
    EmbeddingSaturated,
}

impl DownstreamAdmissionRejectReason {
    #[must_use]
    /// Returns the stable reason token.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LlmSaturated => "llm_saturated",
            Self::EmbeddingSaturated => "embedding_saturated",
        }
    }

    #[must_use]
    /// Returns the user-visible rejection message.
    pub const fn user_message(self) -> &'static str {
        match self {
            Self::LlmSaturated => {
                "System is currently busy with generation traffic. Please retry in a few seconds."
            }
            Self::EmbeddingSaturated => {
                "System memory pipeline is currently busy. Please retry in a few seconds."
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Test-facing downstream in-flight concurrency snapshot.
pub struct DownstreamInFlightSnapshot {
    pub max_in_flight: usize,
    pub available_permits: usize,
    pub in_flight: usize,
    pub saturation_pct: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Test-facing runtime snapshot for all downstream resources.
pub struct DownstreamRuntimeSnapshot {
    pub llm: Option<DownstreamInFlightSnapshot>,
    pub embedding: Option<DownstreamInFlightSnapshot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Test-facing downstream admission metrics snapshot.
pub struct DownstreamAdmissionMetricsSnapshot {
    pub total: u64,
    pub admitted: u64,
    pub rejected: u64,
    pub rejected_llm_saturated: u64,
    pub rejected_embedding_saturated: u64,
    pub reject_rate_pct: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Test-facing downstream admission runtime snapshot.
pub struct DownstreamAdmissionRuntimeSnapshot {
    pub enabled: bool,
    pub llm_reject_threshold_pct: u8,
    pub embedding_reject_threshold_pct: u8,
    pub metrics: DownstreamAdmissionMetricsSnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Test-facing downstream admission decision.
pub struct DownstreamAdmissionDecision {
    pub admitted: bool,
    pub reason: Option<DownstreamAdmissionRejectReason>,
    pub snapshot: DownstreamRuntimeSnapshot,
    pub llm_reject_threshold_pct: u8,
    pub embedding_reject_threshold_pct: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Test-facing downstream admission policy.
pub struct DownstreamAdmissionPolicy {
    pub enabled: bool,
    pub llm_reject_threshold_pct: u8,
    pub embedding_reject_threshold_pct: u8,
}

#[derive(Default)]
/// Test-facing downstream admission metrics accumulator.
pub struct DownstreamAdmissionMetrics {
    inner: internal::DownstreamAdmissionMetrics,
}

impl DownstreamAdmissionMetrics {
    /// Records one admission decision.
    pub fn observe(&self, decision: DownstreamAdmissionDecision) {
        self.inner.observe(to_internal_decision(decision));
    }

    #[must_use]
    /// Returns the current metrics snapshot.
    pub fn snapshot(&self) -> DownstreamAdmissionMetricsSnapshot {
        from_internal_metrics_snapshot(self.inner.snapshot())
    }
}

impl DownstreamAdmissionPolicy {
    #[must_use]
    /// Builds policy from an injected environment lookup.
    pub fn from_lookup<F>(lookup: F) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        from_internal_policy(internal::DownstreamAdmissionPolicy::from_lookup(lookup))
    }

    #[must_use]
    /// Evaluates one runtime snapshot against this policy.
    pub fn evaluate(self, snapshot: DownstreamRuntimeSnapshot) -> DownstreamAdmissionDecision {
        from_internal_decision(
            to_internal_policy(self).evaluate(to_internal_runtime_snapshot(snapshot)),
        )
    }

    #[must_use]
    /// Builds a runtime snapshot by combining policy and metrics.
    pub fn runtime_snapshot(
        self,
        metrics: DownstreamAdmissionMetricsSnapshot,
    ) -> DownstreamAdmissionRuntimeSnapshot {
        from_internal_runtime_snapshot(
            to_internal_policy(self).runtime_snapshot(to_internal_metrics_snapshot(metrics)),
        )
    }
}

fn from_internal_reason(
    reason: internal::DownstreamAdmissionRejectReason,
) -> DownstreamAdmissionRejectReason {
    match reason {
        internal::DownstreamAdmissionRejectReason::LlmSaturated => {
            DownstreamAdmissionRejectReason::LlmSaturated
        }
        internal::DownstreamAdmissionRejectReason::EmbeddingSaturated => {
            DownstreamAdmissionRejectReason::EmbeddingSaturated
        }
    }
}

fn to_internal_reason(
    reason: DownstreamAdmissionRejectReason,
) -> internal::DownstreamAdmissionRejectReason {
    match reason {
        DownstreamAdmissionRejectReason::LlmSaturated => {
            internal::DownstreamAdmissionRejectReason::LlmSaturated
        }
        DownstreamAdmissionRejectReason::EmbeddingSaturated => {
            internal::DownstreamAdmissionRejectReason::EmbeddingSaturated
        }
    }
}

fn from_internal_in_flight(
    value: internal::DownstreamInFlightSnapshot,
) -> DownstreamInFlightSnapshot {
    DownstreamInFlightSnapshot {
        max_in_flight: value.max_in_flight,
        available_permits: value.available_permits,
        in_flight: value.in_flight,
        saturation_pct: value.saturation_pct,
    }
}

fn to_internal_in_flight(
    value: DownstreamInFlightSnapshot,
) -> internal::DownstreamInFlightSnapshot {
    internal::DownstreamInFlightSnapshot {
        max_in_flight: value.max_in_flight,
        available_permits: value.available_permits,
        in_flight: value.in_flight,
        saturation_pct: value.saturation_pct,
    }
}

fn from_internal_runtime_state(
    value: internal::DownstreamRuntimeSnapshot,
) -> DownstreamRuntimeSnapshot {
    DownstreamRuntimeSnapshot {
        llm: value.llm.map(from_internal_in_flight),
        embedding: value.embedding.map(from_internal_in_flight),
    }
}

fn to_internal_runtime_snapshot(
    value: DownstreamRuntimeSnapshot,
) -> internal::DownstreamRuntimeSnapshot {
    internal::DownstreamRuntimeSnapshot {
        llm: value.llm.map(to_internal_in_flight),
        embedding: value.embedding.map(to_internal_in_flight),
    }
}

fn from_internal_metrics_snapshot(
    value: internal::DownstreamAdmissionMetricsSnapshot,
) -> DownstreamAdmissionMetricsSnapshot {
    DownstreamAdmissionMetricsSnapshot {
        total: value.total,
        admitted: value.admitted,
        rejected: value.rejected,
        rejected_llm_saturated: value.rejected_llm_saturated,
        rejected_embedding_saturated: value.rejected_embedding_saturated,
        reject_rate_pct: value.reject_rate_pct,
    }
}

fn to_internal_metrics_snapshot(
    value: DownstreamAdmissionMetricsSnapshot,
) -> internal::DownstreamAdmissionMetricsSnapshot {
    internal::DownstreamAdmissionMetricsSnapshot {
        total: value.total,
        admitted: value.admitted,
        rejected: value.rejected,
        rejected_llm_saturated: value.rejected_llm_saturated,
        rejected_embedding_saturated: value.rejected_embedding_saturated,
        reject_rate_pct: value.reject_rate_pct,
    }
}

fn from_internal_decision(
    value: internal::DownstreamAdmissionDecision,
) -> DownstreamAdmissionDecision {
    DownstreamAdmissionDecision {
        admitted: value.admitted,
        reason: value.reason.map(from_internal_reason),
        snapshot: from_internal_runtime_state(value.snapshot),
        llm_reject_threshold_pct: value.llm_reject_threshold_pct,
        embedding_reject_threshold_pct: value.embedding_reject_threshold_pct,
    }
}

fn to_internal_decision(
    value: DownstreamAdmissionDecision,
) -> internal::DownstreamAdmissionDecision {
    internal::DownstreamAdmissionDecision {
        admitted: value.admitted,
        reason: value.reason.map(to_internal_reason),
        snapshot: to_internal_runtime_snapshot(value.snapshot),
        llm_reject_threshold_pct: value.llm_reject_threshold_pct,
        embedding_reject_threshold_pct: value.embedding_reject_threshold_pct,
    }
}

fn from_internal_policy(value: internal::DownstreamAdmissionPolicy) -> DownstreamAdmissionPolicy {
    DownstreamAdmissionPolicy {
        enabled: value.enabled,
        llm_reject_threshold_pct: value.llm_reject_threshold_pct,
        embedding_reject_threshold_pct: value.embedding_reject_threshold_pct,
    }
}

fn to_internal_policy(value: DownstreamAdmissionPolicy) -> internal::DownstreamAdmissionPolicy {
    internal::DownstreamAdmissionPolicy {
        enabled: value.enabled,
        llm_reject_threshold_pct: value.llm_reject_threshold_pct,
        embedding_reject_threshold_pct: value.embedding_reject_threshold_pct,
    }
}

fn from_internal_runtime_snapshot(
    value: internal::DownstreamAdmissionRuntimeSnapshot,
) -> DownstreamAdmissionRuntimeSnapshot {
    DownstreamAdmissionRuntimeSnapshot {
        enabled: value.enabled,
        llm_reject_threshold_pct: value.llm_reject_threshold_pct,
        embedding_reject_threshold_pct: value.embedding_reject_threshold_pct,
        metrics: from_internal_metrics_snapshot(value.metrics),
    }
}
