//! Profile-aware Julia scheduling decision policy.

use crate::{
    AdmissionDecision, BenchmarkState, ContractValidationState, LaneCapability,
    ManifestReadinessState, PressureLevel, QueueReason, RejectionReason, WarmupState,
};

use super::model::{
    JuliaComputeTaskShape, JuliaRuntimeStats, JuliaScheduleAction, JuliaScheduleBatchabilityKey,
    JuliaSchedulePlan, JuliaScheduleReason, JuliaSchedulingInput, JuliaTaskComplexityClass,
};

const DEFAULT_TARGET_LATENCY_MS: u32 = 250;
const LOW_CONFIDENCE_MARGIN: i32 = 20;
const UNSTABLE_ERROR_RATE_BPS: u32 = 1_000;

impl JuliaSchedulingInput {
    /// Computes an inert Julia scheduling plan.
    #[must_use]
    pub fn plan(&self) -> JuliaSchedulePlan {
        let scores = JuliaScheduleScores::from_input(self);
        let predicted_latency_ms = self.predicted_latency_ms();
        let max_in_flight_recommendation = self.max_in_flight_recommendation();
        let selected_batch_size = self.selected_batch_size();
        let pressure = self.readiness.pressure_level();
        let readiness = self.readiness.readiness_state();

        if let Some(reason) = self.blocking_failure_reason() {
            return self.fallback_or_reject(reason, scores, predicted_latency_ms);
        }

        if self.runtime_is_unstable() {
            return self.fallback_or_reject(
                JuliaScheduleReason::RuntimeUnstable,
                scores,
                predicted_latency_ms,
            );
        }

        match self.readiness.to_admission_budget().decide() {
            AdmissionDecision::Allow { .. } => {
                if matches!(pressure, PressureLevel::High) {
                    return self.queue_or_deadline_fallback(
                        JuliaScheduleReason::QueuePressure,
                        scores,
                        predicted_latency_ms,
                    );
                }
                if scores.confidence < LOW_CONFIDENCE_MARGIN {
                    return self.low_confidence_plan(scores, predicted_latency_ms);
                }
                JuliaSchedulePlan {
                    action: JuliaScheduleAction::Dispatch,
                    reason: JuliaScheduleReason::JuliaAdvantage,
                    capability: self.readiness.capability,
                    profile_id: self.readiness.profile_id.clone().into(),
                    readiness,
                    pressure,
                    selected_batch_size,
                    max_in_flight_recommendation,
                    fallback_available: self.fallback_available,
                    predicted_latency_ms: predicted_latency_ms.into(),
                    benefit_score: scores.benefit,
                    cost_score: scores.cost,
                    confidence_score: scores.confidence,
                    batchability_key: schedule_batchability_key(&self.task_shape),
                }
            }
            AdmissionDecision::Queue { reason, .. } => {
                let reason = match reason {
                    QueueReason::NotReady => JuliaScheduleReason::JuliaWarming,
                    QueueReason::AtCapacity => JuliaScheduleReason::JuliaAtCapacity,
                };
                self.queue_or_deadline_fallback(reason, scores, predicted_latency_ms)
            }
            AdmissionDecision::Reject { reason, .. } => {
                let reason = match reason {
                    RejectionReason::LaneDisabled => JuliaScheduleReason::ContractInvalid,
                    RejectionReason::PressureCritical => JuliaScheduleReason::QueuePressure,
                    RejectionReason::NoCapacity => JuliaScheduleReason::NoCapacity,
                };
                self.fallback_or_reject(reason, scores, predicted_latency_ms)
            }
        }
    }

    fn blocking_failure_reason(&self) -> Option<JuliaScheduleReason> {
        if !self.readiness.capability.owning_lane().is_julia_compute() {
            return Some(JuliaScheduleReason::ContractInvalid);
        }
        if matches!(
            self.readiness.route_validation,
            ContractValidationState::Invalid
        ) || matches!(
            self.readiness.schema_validation,
            ContractValidationState::Invalid
        ) || matches!(
            self.readiness.manifest_readiness,
            ManifestReadinessState::Missing | ManifestReadinessState::Incompatible
        ) {
            return Some(JuliaScheduleReason::ContractInvalid);
        }
        if matches!(self.readiness.benchmark, BenchmarkState::Failed)
            || matches!(self.runtime_stats.benchmark, BenchmarkState::Failed)
        {
            return Some(JuliaScheduleReason::BenchmarkFailed);
        }
        if matches!(self.readiness.warmup, WarmupState::Failed)
            || matches!(self.runtime_stats.warmup, WarmupState::Failed)
        {
            return Some(JuliaScheduleReason::RuntimeUnstable);
        }
        None
    }

    fn runtime_is_unstable(&self) -> bool {
        let target_latency = self.target_latency_ms.unwrap_or(DEFAULT_TARGET_LATENCY_MS);
        let hard_latency_failure = self
            .runtime_stats
            .p95_latency_ms
            .is_some_and(|p95_latency_ms| p95_latency_ms > target_latency.saturating_mul(4));
        self.runtime_stats.error_rate_basis_points >= UNSTABLE_ERROR_RATE_BPS
            || hard_latency_failure
    }

    fn queue_or_deadline_fallback(
        &self,
        reason: JuliaScheduleReason,
        scores: JuliaScheduleScores,
        predicted_latency_ms: u32,
    ) -> JuliaSchedulePlan {
        if self.deadline_is_tight(predicted_latency_ms) && self.fallback_available {
            return self.fallback_plan(
                JuliaScheduleReason::DeadlineTooTight,
                scores,
                predicted_latency_ms,
            );
        }
        self.queue_plan(reason, scores, predicted_latency_ms)
    }

    fn low_confidence_plan(
        &self,
        scores: JuliaScheduleScores,
        predicted_latency_ms: u32,
    ) -> JuliaSchedulePlan {
        if self.fallback_available {
            return self.fallback_plan(
                JuliaScheduleReason::CostExceedsBenefit,
                scores,
                predicted_latency_ms,
            );
        }
        self.queue_plan(
            JuliaScheduleReason::CostExceedsBenefit,
            scores,
            predicted_latency_ms,
        )
    }

    fn fallback_or_reject(
        &self,
        reason: JuliaScheduleReason,
        scores: JuliaScheduleScores,
        predicted_latency_ms: u32,
    ) -> JuliaSchedulePlan {
        if self.fallback_available {
            self.fallback_plan(reason, scores, predicted_latency_ms)
        } else {
            self.reject_plan(reason, scores, predicted_latency_ms)
        }
    }

    fn fallback_plan(
        &self,
        reason: JuliaScheduleReason,
        scores: JuliaScheduleScores,
        predicted_latency_ms: u32,
    ) -> JuliaSchedulePlan {
        self.plan_with_action(
            JuliaScheduleAction::Fallback,
            reason,
            0,
            scores,
            predicted_latency_ms,
        )
    }

    fn queue_plan(
        &self,
        reason: JuliaScheduleReason,
        scores: JuliaScheduleScores,
        predicted_latency_ms: u32,
    ) -> JuliaSchedulePlan {
        self.plan_with_action(
            JuliaScheduleAction::Queue,
            reason,
            0,
            scores,
            predicted_latency_ms,
        )
    }

    fn reject_plan(
        &self,
        reason: JuliaScheduleReason,
        scores: JuliaScheduleScores,
        predicted_latency_ms: u32,
    ) -> JuliaSchedulePlan {
        self.plan_with_action(
            JuliaScheduleAction::Reject,
            reason,
            0,
            scores,
            predicted_latency_ms,
        )
    }

    fn plan_with_action(
        &self,
        action: JuliaScheduleAction,
        reason: JuliaScheduleReason,
        selected_batch_size: u32,
        scores: JuliaScheduleScores,
        predicted_latency_ms: u32,
    ) -> JuliaSchedulePlan {
        JuliaSchedulePlan {
            action,
            reason,
            capability: self.readiness.capability,
            profile_id: self.readiness.profile_id.clone().into(),
            readiness: self.readiness.readiness_state(),
            pressure: self.readiness.pressure_level(),
            selected_batch_size,
            max_in_flight_recommendation: self.max_in_flight_recommendation(),
            fallback_available: self.fallback_available,
            predicted_latency_ms: predicted_latency_ms.into(),
            benefit_score: scores.benefit,
            cost_score: scores.cost,
            confidence_score: scores.confidence,
            batchability_key: schedule_batchability_key(&self.task_shape),
        }
    }

    fn max_in_flight_recommendation(&self) -> u32 {
        self.readiness
            .max_in_flight
            .unwrap_or_else(|| profile_default_max_in_flight(self.readiness.capability))
            .max(1)
    }

    fn selected_batch_size(&self) -> u32 {
        if self.task_shape.batchability_key.is_none() {
            return 1;
        }
        let rows = self.task_shape.normalized_rows();
        rows.min(self.task_shape.complexity.batch_cap())
            .min(self.max_in_flight_recommendation())
            .max(1)
    }

    fn predicted_latency_ms(&self) -> u32 {
        let profile_latency = self
            .runtime_stats
            .p95_latency_ms
            .unwrap_or_else(|| profile_default_p95_latency_ms(self.readiness.capability));
        profile_latency
            .saturating_add(self.transfer_cost_ms())
            .saturating_add(self.runtime_stats.queue_depth.saturating_mul(12))
    }

    fn deadline_is_tight(&self, predicted_latency_ms: u32) -> bool {
        self.deadline_ms
            .is_some_and(|deadline_ms| predicted_latency_ms >= deadline_ms)
    }

    fn transfer_cost_ms(&self) -> u32 {
        bytes_to_units(self.task_shape.byte_size, 512 * 1024).min(120)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct JuliaScheduleScores {
    benefit: i32,
    cost: i32,
    confidence: i32,
}

impl JuliaScheduleScores {
    fn from_input(input: &JuliaSchedulingInput) -> Self {
        let benefit = benefit_score(input);
        let cost = cost_score(input);
        Self {
            benefit,
            cost,
            confidence: benefit - cost,
        }
    }
}

fn benefit_score(input: &JuliaSchedulingInput) -> i32 {
    profile_base_benefit(input.readiness.capability)
        + input.task_shape.complexity.benefit_bonus()
        + shape_benefit(&input.task_shape)
        + batch_benefit(&input.task_shape)
}

fn cost_score(input: &JuliaSchedulingInput) -> i32 {
    let target_latency = input.target_latency_ms.unwrap_or(DEFAULT_TARGET_LATENCY_MS);
    transfer_cost_score(&input.task_shape)
        + queue_cost_score(input.runtime_stats)
        + latency_cost_score(input.runtime_stats, target_latency)
        + stability_cost_score(input.runtime_stats)
        + rust_fallback_preference_cost(input)
}

const fn profile_base_benefit(capability: LaneCapability) -> i32 {
    match capability {
        LaneCapability::GraphEvidenceCompute => 70,
        LaneCapability::GraphSearchCompute => 80,
        LaneCapability::ScientificCompute => 90,
        LaneCapability::MemoryProfileCompute => 45,
        LaneCapability::DocumentExtraction | LaneCapability::OcrShardExtraction => 0,
    }
}

fn shape_benefit(shape: &JuliaComputeTaskShape) -> i32 {
    capped_i32(shape.rows / 100 * 4, 80)
        + capped_i32(shape.nodes / 100 * 5, 100)
        + capped_i32(shape.edges / 500 * 5, 100)
        + capped_i32(shape.feature_columns.saturating_mul(2), 40)
        + capped_i32(bytes_to_units(shape.byte_size, 1024 * 1024) * 5, 50)
}

fn batch_benefit(shape: &JuliaComputeTaskShape) -> i32 {
    if shape.batchability_key.is_none() {
        return 0;
    }
    if shape.normalized_rows() >= 16 {
        40
    } else {
        30
    }
}

fn schedule_batchability_key(
    shape: &JuliaComputeTaskShape,
) -> Option<JuliaScheduleBatchabilityKey> {
    shape.batchability_key.clone().map(Into::into)
}

fn transfer_cost_score(shape: &JuliaComputeTaskShape) -> i32 {
    capped_i32(bytes_to_units(shape.byte_size, 256 * 1024) * 3, 120)
}

fn rust_fallback_preference_cost(input: &JuliaSchedulingInput) -> i32 {
    if !input.fallback_available
        || !matches!(
            input.task_shape.complexity,
            JuliaTaskComplexityClass::Simple
        )
    {
        return 0;
    }
    match input.readiness.capability {
        LaneCapability::GraphEvidenceCompute
        | LaneCapability::GraphSearchCompute
        | LaneCapability::MemoryProfileCompute => 80,
        LaneCapability::ScientificCompute
        | LaneCapability::DocumentExtraction
        | LaneCapability::OcrShardExtraction => 0,
    }
}

fn queue_cost_score(stats: JuliaRuntimeStats) -> i32 {
    capped_i32(
        stats
            .queue_depth
            .saturating_mul(12)
            .saturating_add(stats.active_in_flight.saturating_mul(4)),
        180,
    )
}

fn latency_cost_score(stats: JuliaRuntimeStats, target_latency_ms: u32) -> i32 {
    let p95 = stats.p95_latency_ms.unwrap_or(target_latency_ms);
    let baseline = p95 / 10;
    let target_excess = p95.saturating_sub(target_latency_ms) / 5;
    capped_i32(baseline.saturating_add(target_excess), 160)
}

fn stability_cost_score(stats: JuliaRuntimeStats) -> i32 {
    let warmup = match stats.warmup {
        WarmupState::Ready => 0,
        WarmupState::Cold | WarmupState::Warming => 80,
        WarmupState::Unknown => 40,
        WarmupState::Failed => 200,
    };
    let benchmark = match stats.benchmark {
        BenchmarkState::WithinThreshold | BenchmarkState::NotRequired => 0,
        BenchmarkState::Unknown => 20,
        BenchmarkState::AboveThreshold => 50,
        BenchmarkState::Failed => 200,
    };
    let error = stats.error_rate_basis_points / 20;
    capped_i32(warmup + benchmark + error, 260)
}

const fn profile_default_max_in_flight(capability: LaneCapability) -> u32 {
    match capability {
        LaneCapability::GraphSearchCompute => 8,
        LaneCapability::GraphEvidenceCompute
        | LaneCapability::ScientificCompute
        | LaneCapability::MemoryProfileCompute => 4,
        LaneCapability::DocumentExtraction | LaneCapability::OcrShardExtraction => 1,
    }
}

const fn profile_default_p95_latency_ms(capability: LaneCapability) -> u32 {
    match capability {
        LaneCapability::GraphEvidenceCompute => 45,
        LaneCapability::GraphSearchCompute => 65,
        LaneCapability::ScientificCompute => 90,
        LaneCapability::MemoryProfileCompute => 25,
        LaneCapability::DocumentExtraction | LaneCapability::OcrShardExtraction => {
            DEFAULT_TARGET_LATENCY_MS
        }
    }
}

fn bytes_to_units(bytes: u64, unit: u64) -> u32 {
    if bytes == 0 {
        return 0;
    }
    let units = bytes.div_ceil(unit);
    u32::try_from(units).unwrap_or(u32::MAX)
}

fn capped_i32(value: u32, cap: i32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX).min(cap)
}
