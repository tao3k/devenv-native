//! Memory persistence helpers for state snapshots, stream publishing, and decay.

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Instant;

use xiuxian_memory_engine::{EpisodeStore, MemoryGatePolicy};

use crate::observability::SessionEvent;

use crate::agent::Agent;
use crate::agent::memory::{sanitize_decay_factor, should_apply_decay};

pub(in crate::agent) fn persist_memory_state(
    backend: Option<&Arc<crate::agent::memory_state::MemoryStateBackend>>,
    store: &EpisodeStore,
    session_id: &str,
    reason: &str,
) {
    let Some(backend) = backend else {
        return;
    };

    match backend.save(store) {
        Ok(()) => {
            tracing::debug!(
                event = SessionEvent::MemoryStateSaveSucceeded.as_str(),
                session_id,
                reason,
                "memory state persisted"
            );
        }
        Err(error) => {
            tracing::warn!(
                event = SessionEvent::MemoryStateSaveFailed.as_str(),
                session_id,
                reason,
                error = %error,
                "failed to persist memory state"
            );
        }
    }
}

impl Agent {
    pub(in crate::agent) fn memory_gate_policy(&self) -> MemoryGatePolicy {
        let mut policy = MemoryGatePolicy::default();
        let Some(memory_cfg) = self.config.memory.as_ref() else {
            return policy;
        };

        policy.promote_threshold = memory_cfg.gate_promote_threshold.clamp(0.0, 1.0);
        policy.obsolete_threshold = memory_cfg.gate_obsolete_threshold.clamp(0.0, 1.0);
        policy.promote_min_usage = memory_cfg.gate_promote_min_usage.max(1);
        policy.obsolete_min_usage = memory_cfg.gate_obsolete_min_usage.max(1);
        policy.promote_failure_rate_ceiling =
            memory_cfg.gate_promote_failure_rate_ceiling.clamp(0.0, 1.0);
        policy.obsolete_failure_rate_floor =
            memory_cfg.gate_obsolete_failure_rate_floor.clamp(0.0, 1.0);
        policy.promote_min_ttl_score = memory_cfg.gate_promote_min_ttl_score.clamp(0.0, 1.0);
        policy.obsolete_max_ttl_score = memory_cfg.gate_obsolete_max_ttl_score.clamp(0.0, 1.0);
        policy
    }

    pub(in crate::agent) fn memory_stream_name(&self) -> &str {
        self.config
            .memory
            .as_ref()
            .map(|cfg| cfg.stream_name.trim())
            .filter(|value| !value.is_empty())
            .unwrap_or("memory.events")
    }

    pub(in crate::agent) async fn publish_memory_stream_event(
        &self,
        fields: Vec<(String, String)>,
    ) {
        if let Err(error) = self
            .session
            .publish_stream_event(self.memory_stream_name(), fields)
            .await
        {
            tracing::warn!(
                error = %error,
                "failed to publish memory stream event"
            );
        }
    }

    pub(in crate::agent) fn maybe_apply_memory_decay(
        &self,
        session_id: &str,
        store: &EpisodeStore,
    ) {
        let Some(memory_cfg) = self.config.memory.as_ref() else {
            return;
        };
        let turn_index = self
            .memory_decay_turn_counter
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1);
        if !should_apply_decay(
            memory_cfg.decay_enabled,
            memory_cfg.decay_every_turns,
            turn_index,
        ) {
            return;
        }
        let decay_factor = sanitize_decay_factor(memory_cfg.decay_factor);
        let started = Instant::now();
        store.apply_decay(decay_factor);
        persist_memory_state(
            self.memory_state_backend.as_ref(),
            store,
            session_id,
            "decay",
        );
        tracing::debug!(
            event = SessionEvent::MemoryDecayApplied.as_str(),
            session_id,
            turn_index,
            decay_every_turns = memory_cfg.decay_every_turns,
            decay_factor,
            duration_ms = started.elapsed().as_millis(),
            "memory decay applied"
        );
    }
}
