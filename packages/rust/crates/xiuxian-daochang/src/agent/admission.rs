use crate::embedding::{EmbeddingClient, EmbeddingInFlightSnapshot};
use crate::llm::LlmInFlightSnapshot;
use std::sync::atomic::{AtomicU64, Ordering};

use super::Agent;

const ADMISSION_ENABLED_ENV: &str = "XIUXIAN_DAOCHANG_DOWNSTREAM_ADMISSION_ENABLED";
const ADMISSION_LLM_THRESHOLD_ENV: &str = "XIUXIAN_DAOCHANG_ADMISSION_LLM_SATURATION_PCT";
const ADMISSION_EMBED_THRESHOLD_ENV: &str = "XIUXIAN_DAOCHANG_ADMISSION_EMBED_SATURATION_PCT";
const DEFAULT_ADMISSION_ENABLED: bool = true;
const DEFAULT_ADMISSION_LLM_THRESHOLD_PCT: u8 = 95;
const DEFAULT_ADMISSION_EMBED_THRESHOLD_PCT: u8 = 95;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DownstreamInFlightSnapshot {
    pub max_in_flight: usize,
    pub available_permits: usize,
    pub in_flight: usize,
    pub saturation_pct: u8,
}

impl From<LlmInFlightSnapshot> for DownstreamInFlightSnapshot {
    fn from(value: LlmInFlightSnapshot) -> Self {
        Self {
            max_in_flight: value.max_in_flight,
            available_permits: value.available_permits,
            in_flight: value.in_flight,
            saturation_pct: value.saturation_pct,
        }
    }
}

impl From<EmbeddingInFlightSnapshot> for DownstreamInFlightSnapshot {
    fn from(value: EmbeddingInFlightSnapshot) -> Self {
        Self {
            max_in_flight: value.max_in_flight,
            available_permits: value.available_permits,
            in_flight: value.in_flight,
            saturation_pct: value.saturation_pct,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct DownstreamRuntimeSnapshot {
    pub llm: Option<DownstreamInFlightSnapshot>,
    pub embedding: Option<DownstreamInFlightSnapshot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct DownstreamAdmissionMetricsSnapshot {
    pub total: u64,
    pub admitted: u64,
    pub rejected: u64,
    pub rejected_llm_saturated: u64,
    pub rejected_embedding_saturated: u64,
    pub reject_rate_pct: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DownstreamAdmissionRuntimeSnapshot {
    pub enabled: bool,
    pub llm_reject_threshold_pct: u8,
    pub embedding_reject_threshold_pct: u8,
    pub metrics: DownstreamAdmissionMetricsSnapshot,
}

#[derive(Debug, Default)]
pub(crate) struct DownstreamAdmissionMetrics {
    total: AtomicU64,
    admitted: AtomicU64,
    rejected: AtomicU64,
    rejected_llm_saturated: AtomicU64,
    rejected_embedding_saturated: AtomicU64,
}

impl DownstreamAdmissionMetrics {
    pub(crate) fn observe(&self, decision: DownstreamAdmissionDecision) {
        self.total.fetch_add(1, Ordering::Relaxed);
        if decision.admitted {
            self.admitted.fetch_add(1, Ordering::Relaxed);
            return;
        }
        self.rejected.fetch_add(1, Ordering::Relaxed);
        match decision.reason {
            Some(DownstreamAdmissionRejectReason::LlmSaturated) => {
                self.rejected_llm_saturated.fetch_add(1, Ordering::Relaxed);
            }
            Some(DownstreamAdmissionRejectReason::EmbeddingSaturated) => {
                self.rejected_embedding_saturated
                    .fetch_add(1, Ordering::Relaxed);
            }
            None => {}
        }
    }

    pub(crate) fn snapshot(&self) -> DownstreamAdmissionMetricsSnapshot {
        let total = self.total.load(Ordering::Relaxed);
        let admitted = self.admitted.load(Ordering::Relaxed);
        let rejected = self.rejected.load(Ordering::Relaxed);
        let rejected_llm_saturated = self.rejected_llm_saturated.load(Ordering::Relaxed);
        let rejected_embedding_saturated =
            self.rejected_embedding_saturated.load(Ordering::Relaxed);
        DownstreamAdmissionMetricsSnapshot {
            total,
            admitted,
            rejected,
            rejected_llm_saturated,
            rejected_embedding_saturated,
            reject_rate_pct: ratio_as_pct(rejected, total),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DownstreamAdmissionRejectReason {
    LlmSaturated,
    EmbeddingSaturated,
}

impl DownstreamAdmissionRejectReason {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::LlmSaturated => "llm_saturated",
            Self::EmbeddingSaturated => "embedding_saturated",
        }
    }

    pub(crate) fn user_message(self) -> &'static str {
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
pub(crate) struct DownstreamAdmissionDecision {
    pub admitted: bool,
    pub reason: Option<DownstreamAdmissionRejectReason>,
    pub snapshot: DownstreamRuntimeSnapshot,
    pub llm_reject_threshold_pct: u8,
    pub embedding_reject_threshold_pct: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DownstreamAdmissionPolicy {
    pub enabled: bool,
    pub llm_reject_threshold_pct: u8,
    pub embedding_reject_threshold_pct: u8,
}

impl DownstreamAdmissionPolicy {
    pub(crate) fn from_env() -> Self {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    pub(crate) fn from_lookup<F>(lookup: F) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        let enabled =
            parse_bool_env(&lookup, ADMISSION_ENABLED_ENV).unwrap_or(DEFAULT_ADMISSION_ENABLED);
        let llm_reject_threshold_pct = parse_threshold_env(
            &lookup,
            ADMISSION_LLM_THRESHOLD_ENV,
            DEFAULT_ADMISSION_LLM_THRESHOLD_PCT,
        );
        let embedding_reject_threshold_pct = parse_threshold_env(
            &lookup,
            ADMISSION_EMBED_THRESHOLD_ENV,
            DEFAULT_ADMISSION_EMBED_THRESHOLD_PCT,
        );
        tracing::info!(
            event = "agent.admission.policy",
            enabled,
            llm_reject_threshold_pct,
            embedding_reject_threshold_pct,
            "downstream admission policy configured"
        );
        Self {
            enabled,
            llm_reject_threshold_pct,
            embedding_reject_threshold_pct,
        }
    }

    pub(crate) fn evaluate(
        self,
        snapshot: DownstreamRuntimeSnapshot,
    ) -> DownstreamAdmissionDecision {
        if !self.enabled {
            return DownstreamAdmissionDecision {
                admitted: true,
                reason: None,
                snapshot,
                llm_reject_threshold_pct: self.llm_reject_threshold_pct,
                embedding_reject_threshold_pct: self.embedding_reject_threshold_pct,
            };
        }
        if snapshot
            .llm
            .is_some_and(|state| state.saturation_pct >= self.llm_reject_threshold_pct)
        {
            return DownstreamAdmissionDecision {
                admitted: false,
                reason: Some(DownstreamAdmissionRejectReason::LlmSaturated),
                snapshot,
                llm_reject_threshold_pct: self.llm_reject_threshold_pct,
                embedding_reject_threshold_pct: self.embedding_reject_threshold_pct,
            };
        }
        if snapshot
            .embedding
            .is_some_and(|state| state.saturation_pct >= self.embedding_reject_threshold_pct)
        {
            return DownstreamAdmissionDecision {
                admitted: false,
                reason: Some(DownstreamAdmissionRejectReason::EmbeddingSaturated),
                snapshot,
                llm_reject_threshold_pct: self.llm_reject_threshold_pct,
                embedding_reject_threshold_pct: self.embedding_reject_threshold_pct,
            };
        }
        DownstreamAdmissionDecision {
            admitted: true,
            reason: None,
            snapshot,
            llm_reject_threshold_pct: self.llm_reject_threshold_pct,
            embedding_reject_threshold_pct: self.embedding_reject_threshold_pct,
        }
    }

    pub(crate) fn runtime_snapshot(
        self,
        metrics: DownstreamAdmissionMetricsSnapshot,
    ) -> DownstreamAdmissionRuntimeSnapshot {
        DownstreamAdmissionRuntimeSnapshot {
            enabled: self.enabled,
            llm_reject_threshold_pct: self.llm_reject_threshold_pct,
            embedding_reject_threshold_pct: self.embedding_reject_threshold_pct,
            metrics,
        }
    }
}

fn parse_bool_env<F>(lookup: &F, name: &str) -> Option<bool>
where
    F: Fn(&str) -> Option<String>,
{
    let raw = lookup(name)?;
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => {
            tracing::warn!(env_var = %name, value = %raw, "invalid boolean env value");
            None
        }
    }
}

fn parse_threshold_env<F>(lookup: &F, name: &str, default: u8) -> u8
where
    F: Fn(&str) -> Option<String>,
{
    let Some(raw) = lookup(name) else {
        return default;
    };
    if let Ok(value @ 1..=100) = raw.trim().parse::<u8>() {
        value
    } else {
        tracing::warn!(
            env_var = %name,
            value = %raw,
            default,
            "invalid admission threshold env value; using default"
        );
        default
    }
}

fn ratio_as_pct(numerator: u64, denominator: u64) -> u8 {
    if denominator == 0 {
        return 0;
    }
    let value = numerator.saturating_mul(100) / denominator;
    u8::try_from(value).unwrap_or(100).min(100)
}

impl Agent {
    pub(crate) fn evaluate_downstream_admission(&self) -> DownstreamAdmissionDecision {
        let snapshot = DownstreamRuntimeSnapshot {
            llm: self.llm.in_flight_snapshot().map(Into::into),
            embedding: self
                .embedding_client
                .as_ref()
                .and_then(EmbeddingClient::in_flight_snapshot)
                .map(Into::into),
        };
        let decision = self.downstream_admission_policy.evaluate(snapshot);
        self.downstream_admission_metrics.observe(decision);
        decision
    }

    pub(crate) fn downstream_admission_runtime_snapshot(
        &self,
    ) -> DownstreamAdmissionRuntimeSnapshot {
        let metrics = self.downstream_admission_metrics.snapshot();
        self.downstream_admission_policy.runtime_snapshot(metrics)
    }
}
