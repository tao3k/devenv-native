//! Rust-side capacity feedback for audio shard analyzer dispatch.

use std::sync::{Mutex, MutexGuard};

use xiuxian_wendao_attachments::polyglot::{
    AudioShardScheduleRequest, audio_shard_pressure_evidence, audio_shard_schedule_plan,
};

const HEALTHY_STREAK_BEFORE_INCREASE: usize = 2;
const LATENCY_PRESSURE_PER_SHARD_MS: u64 = 120_000;
const TARGET_INITIAL_AUDIO_SHARD_WAVES: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AudioShardCapacitySnapshot {
    pub(crate) max_worker_bound: usize,
    pub(crate) current_worker_budget: usize,
    pub(crate) healthy_streak: usize,
    pub(crate) budget_increase_events: u64,
    pub(crate) budget_decrease_events: u64,
}

#[derive(Debug)]
pub(crate) struct AudioShardCapacityController {
    state: Mutex<AudioShardCapacityState>,
}

#[derive(Debug, Clone)]
struct AudioShardCapacityState {
    max_worker_bound: usize,
    current_worker_budget: usize,
    healthy_streak: usize,
    budget_increase_events: u64,
    budget_decrease_events: u64,
}

impl AudioShardCapacityController {
    pub(crate) fn new(max_worker_bound: usize) -> Self {
        let max_worker_bound = max_worker_bound.max(1);
        Self {
            state: Mutex::new(AudioShardCapacityState {
                max_worker_bound,
                current_worker_budget: initial_worker_budget(max_worker_bound),
                healthy_streak: 0,
                budget_increase_events: 0,
                budget_decrease_events: 0,
            }),
        }
    }

    #[cfg(test)]
    pub(crate) fn new_with_current_budget(
        max_worker_bound: usize,
        current_worker_budget: usize,
    ) -> Self {
        let max_worker_bound = max_worker_bound.max(1);
        Self {
            state: Mutex::new(AudioShardCapacityState {
                max_worker_bound,
                current_worker_budget: current_worker_budget.clamp(1, max_worker_bound),
                healthy_streak: 0,
                budget_increase_events: 0,
                budget_decrease_events: 0,
            }),
        }
    }

    pub(crate) fn budget_for_shards(&self, shard_count: usize) -> usize {
        let snapshot = self.snapshot();
        let current_worker_budget = if snapshot.budget_decrease_events == 0 {
            snapshot.current_worker_budget.max(shard_wave_worker_floor(
                shard_count,
                snapshot.max_worker_bound,
            ))
        } else {
            snapshot.current_worker_budget
        };
        scheduled_audio_worker_budget(
            shard_count,
            current_worker_budget,
            snapshot.max_worker_bound,
        )
    }

    pub(crate) fn record_success(&self, shard_count: usize, latency_ms: u64) {
        if latency_ms > audio_capacity_latency_pressure_limit_ms(shard_count) {
            self.record_pressure();
            return;
        }

        let mut state = self.lock_state();
        state.healthy_streak = state.healthy_streak.saturating_add(1);
        if state.healthy_streak >= HEALTHY_STREAK_BEFORE_INCREASE
            && state.current_worker_budget < state.max_worker_bound
        {
            state.current_worker_budget = state.current_worker_budget.saturating_add(1);
            state.healthy_streak = 0;
            state.budget_increase_events = state.budget_increase_events.saturating_add(1);
        }
    }

    pub(crate) fn record_failure(&self) {
        self.record_pressure();
    }

    pub(crate) fn snapshot(&self) -> AudioShardCapacitySnapshot {
        let state = self.lock_state();
        AudioShardCapacitySnapshot {
            max_worker_bound: state.max_worker_bound,
            current_worker_budget: state.current_worker_budget,
            healthy_streak: state.healthy_streak,
            budget_increase_events: state.budget_increase_events,
            budget_decrease_events: state.budget_decrease_events,
        }
    }

    fn record_pressure(&self) {
        let mut state = self.lock_state();
        state.healthy_streak = 0;
        let reduced = state.current_worker_budget.div_ceil(2).max(1);
        if reduced < state.current_worker_budget {
            state.current_worker_budget = reduced;
            state.budget_decrease_events = state.budget_decrease_events.saturating_add(1);
        }
    }

    fn lock_state(&self) -> MutexGuard<'_, AudioShardCapacityState> {
        match self.state.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

pub(crate) fn audio_capacity_latency_pressure_limit_ms(shard_count: usize) -> u64 {
    LATENCY_PRESSURE_PER_SHARD_MS.saturating_mul(shard_count.max(1) as u64)
}

fn scheduled_audio_worker_budget(
    shard_count: usize,
    current_worker_budget: usize,
    max_worker_bound: usize,
) -> usize {
    let shard_count = shard_count.max(1);
    let current_worker_budget = current_worker_budget.max(1);
    let max_worker_bound = max_worker_bound.max(1);
    let pressure = audio_shard_pressure_evidence(
        Some(saturating_usize_to_u32(max_worker_bound)),
        0,
        0,
        0,
        0,
        0,
        false,
    );
    let plan = audio_shard_schedule_plan(AudioShardScheduleRequest {
        pressure,
        adaptive_worker_budget: Some(saturating_usize_to_u32(current_worker_budget)),
        diagnostic_worker_override: None,
        max_worker_cap: Some(saturating_usize_to_u32(max_worker_bound)),
        shard_count: saturating_usize_to_u32(shard_count),
    });
    usize::try_from(plan.recommended_workers)
        .unwrap_or(usize::MAX)
        .max(1)
}

fn saturating_usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn initial_worker_budget(max_worker_bound: usize) -> usize {
    ceil_sqrt_usize(max_worker_bound.max(1)).max(1)
}

fn shard_wave_worker_floor(shard_count: usize, max_worker_bound: usize) -> usize {
    shard_count
        .max(1)
        .div_ceil(TARGET_INITIAL_AUDIO_SHARD_WAVES)
        .clamp(1, max_worker_bound.max(1))
}

fn ceil_sqrt_usize(value: usize) -> usize {
    if value <= 1 {
        return value;
    }
    let mut root = 1usize;
    while root.saturating_mul(root) < value {
        root = root.saturating_add(1);
    }
    root
}

#[cfg(test)]
#[path = "../../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/provider/audio/capacity.rs"]
mod tests;
