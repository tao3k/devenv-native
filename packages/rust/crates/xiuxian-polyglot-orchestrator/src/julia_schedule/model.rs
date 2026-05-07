//! Profile-aware Julia compute scheduling contracts.
//!
//! The scheduler translates owner-supplied readiness, runtime stats, and task
//! shape evidence into an inert plan. It does not call Julia, mutate queues,
//! execute Rust fallback code, or own any Wendao domain algorithm.

use serde::{Deserialize, Serialize};

use crate::{
    AdmissionDecision, BenchmarkState, ContractValidationState, JuliaReadinessEvidence,
    LaneCapability, ManifestReadinessState, PressureLevel, QueueReason, ReadinessState,
    RejectionReason, WarmupState,
};

const DEFAULT_TARGET_LATENCY_MS: u32 = 250;
const LOW_CONFIDENCE_MARGIN: i32 = 20;
const UNSTABLE_ERROR_RATE_BPS: u32 = 1_000;

/// Complexity class supplied by the Julia profile owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JuliaTaskComplexityClass {
    /// Small or structurally simple work that Rust can usually handle well.
    Simple,
    /// Mixed work where Julia may help when warm or batched.
    Balanced,
    /// Numerically or structurally heavy work that should prefer Julia.
    Heavy,
}

impl JuliaTaskComplexityClass {
    const fn benefit_bonus(self) -> i32 {
        match self {
            Self::Simple => 0,
            Self::Balanced => 40,
            Self::Heavy => 90,
        }
    }

    const fn batch_cap(self) -> u32 {
        match self {
            Self::Simple => 4,
            Self::Balanced => 16,
            Self::Heavy => 32,
        }
    }
}

/// Shape evidence for one Julia compute request or batch candidate.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct JuliaComputeTaskShape {
    /// Logical rows or candidate items in the task.
    pub rows: u32,
    /// Graph node count relevant to this task.
    pub nodes: u32,
    /// Graph edge count relevant to this task.
    pub edges: u32,
    /// Feature or signal column count relevant to this task.
    pub feature_columns: u32,
    /// Estimated serialized input bytes.
    pub byte_size: u64,
    /// Optional key proving this task can batch with similar tasks.
    pub batchability_key: Option<String>,
    /// Owner-supplied complexity class.
    pub complexity: JuliaTaskComplexityClass,
}

impl JuliaComputeTaskShape {
    /// Creates an empty shape with balanced complexity.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            rows: 1,
            nodes: 0,
            edges: 0,
            feature_columns: 0,
            byte_size: 0,
            batchability_key: None,
            complexity: JuliaTaskComplexityClass::Balanced,
        }
    }

    /// Returns this shape with row count.
    #[must_use]
    pub const fn with_rows(mut self, rows: u32) -> Self {
        self.rows = rows;
        self
    }

    /// Returns this shape with graph node and edge counts.
    #[must_use]
    pub const fn with_graph_size(mut self, nodes: u32, edges: u32) -> Self {
        self.nodes = nodes;
        self.edges = edges;
        self
    }

    /// Returns this shape with feature column count.
    #[must_use]
    pub const fn with_feature_columns(mut self, feature_columns: u32) -> Self {
        self.feature_columns = feature_columns;
        self
    }

    /// Returns this shape with estimated serialized bytes.
    #[must_use]
    pub const fn with_byte_size(mut self, byte_size: u64) -> Self {
        self.byte_size = byte_size;
        self
    }

    /// Returns this shape with a batchability key.
    #[must_use]
    pub fn with_batchability_key(mut self, batchability_key: impl Into<String>) -> Self {
        self.batchability_key = Some(batchability_key.into());
        self
    }

    /// Returns this shape with complexity class.
    #[must_use]
    pub const fn with_complexity(mut self, complexity: JuliaTaskComplexityClass) -> Self {
        self.complexity = complexity;
        self
    }

    fn normalized_rows(&self) -> u32 {
        self.rows.max(1)
    }
}

impl Default for JuliaComputeTaskShape {
    fn default() -> Self {
        Self::new()
    }
}

/// Runtime evidence observed by the owner package for a Julia profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct JuliaRuntimeStats {
    /// Optional observed p50 latency in milliseconds.
    pub p50_latency_ms: Option<u32>,
    /// Optional observed p95 latency in milliseconds.
    pub p95_latency_ms: Option<u32>,
    /// Recent error rate in basis points.
    pub error_rate_basis_points: u32,
    /// Profile warmup state.
    pub warmup: WarmupState,
    /// Current queue depth for this profile.
    pub queue_depth: u32,
    /// Current active in-flight count for this profile.
    pub active_in_flight: u32,
    /// Profile benchmark state.
    pub benchmark: BenchmarkState,
}

impl JuliaRuntimeStats {
    /// Creates empty runtime stats.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            p50_latency_ms: None,
            p95_latency_ms: None,
            error_rate_basis_points: 0,
            warmup: WarmupState::Unknown,
            queue_depth: 0,
            active_in_flight: 0,
            benchmark: BenchmarkState::Unknown,
        }
    }

    /// Returns this stats record with p50 and p95 latency values.
    #[must_use]
    pub const fn with_latency_ms(
        mut self,
        p50_latency_ms: Option<u32>,
        p95_latency_ms: Option<u32>,
    ) -> Self {
        self.p50_latency_ms = p50_latency_ms;
        self.p95_latency_ms = p95_latency_ms;
        self
    }

    /// Returns this stats record with error rate in basis points.
    #[must_use]
    pub const fn with_error_rate_basis_points(mut self, error_rate_basis_points: u32) -> Self {
        self.error_rate_basis_points = error_rate_basis_points;
        self
    }

    /// Returns this stats record with warmup state.
    #[must_use]
    pub const fn with_warmup(mut self, warmup: WarmupState) -> Self {
        self.warmup = warmup;
        self
    }

    /// Returns this stats record with profile queue counters.
    #[must_use]
    pub const fn with_queue(mut self, queue_depth: u32, active_in_flight: u32) -> Self {
        self.queue_depth = queue_depth;
        self.active_in_flight = active_in_flight;
        self
    }

    /// Returns this stats record with benchmark state.
    #[must_use]
    pub const fn with_benchmark(mut self, benchmark: BenchmarkState) -> Self {
        self.benchmark = benchmark;
        self
    }
}

impl Default for JuliaRuntimeStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Owner-supplied input for a Julia profile scheduling plan.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct JuliaSchedulingInput {
    /// Readiness evidence for the target Julia profile.
    pub readiness: JuliaReadinessEvidence,
    /// Shape evidence for the task or batch candidate.
    pub task_shape: JuliaComputeTaskShape,
    /// Runtime stats observed for the target Julia profile.
    pub runtime_stats: JuliaRuntimeStats,
    /// Whether the owner can run a basic Rust fallback for this task.
    pub fallback_available: bool,
    /// Optional hard deadline in milliseconds.
    pub deadline_ms: Option<u32>,
    /// Optional target latency in milliseconds.
    pub target_latency_ms: Option<u32>,
}

impl JuliaSchedulingInput {
    /// Creates scheduling input from readiness, task shape, and runtime stats.
    #[must_use]
    pub fn new(
        readiness: JuliaReadinessEvidence,
        task_shape: JuliaComputeTaskShape,
        runtime_stats: JuliaRuntimeStats,
    ) -> Self {
        let fallback_available = readiness.fallback_available;
        Self {
            readiness,
            task_shape,
            runtime_stats,
            fallback_available,
            deadline_ms: None,
            target_latency_ms: None,
        }
    }

    /// Returns this input with fallback availability.
    #[must_use]
    pub const fn with_fallback_available(mut self, fallback_available: bool) -> Self {
        self.fallback_available = fallback_available;
        self
    }

    /// Returns this input with a hard deadline in milliseconds.
    #[must_use]
    pub const fn with_deadline_ms(mut self, deadline_ms: Option<u32>) -> Self {
        self.deadline_ms = deadline_ms;
        self
    }

    /// Returns this input with a target latency in milliseconds.
    #[must_use]
    pub const fn with_target_latency_ms(mut self, target_latency_ms: Option<u32>) -> Self {
        self.target_latency_ms = target_latency_ms;
        self
    }

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
                    profile_id: self.readiness.profile_id.clone(),
                    readiness,
                    pressure,
                    selected_batch_size,
                    max_in_flight_recommendation,
                    fallback_available: self.fallback_available,
                    predicted_latency_ms,
                    benefit_score: scores.benefit,
                    cost_score: scores.cost,
                    confidence_score: scores.confidence,
                    batchability_key: self.task_shape.batchability_key.clone(),
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
            profile_id: self.readiness.profile_id.clone(),
            readiness: self.readiness.readiness_state(),
            pressure: self.readiness.pressure_level(),
            selected_batch_size,
            max_in_flight_recommendation: self.max_in_flight_recommendation(),
            fallback_available: self.fallback_available,
            predicted_latency_ms,
            benefit_score: scores.benefit,
            cost_score: scores.cost,
            confidence_score: scores.confidence,
            batchability_key: self.task_shape.batchability_key.clone(),
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

/// Scheduling action recommended for a Julia profile request.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JuliaScheduleAction {
    /// Dispatch work to the Julia profile owner.
    Dispatch,
    /// Keep work queued in the owner package.
    Queue,
    /// Use the owner package's Rust fallback implementation.
    Fallback,
    /// Reject because neither Julia nor fallback is safe.
    Reject,
}

/// Reason attached to a Julia scheduling action.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JuliaScheduleReason {
    /// Julia is expected to beat Rust fallback for this profile and shape.
    JuliaAdvantage,
    /// Julia evidence is still warming or unknown.
    JuliaWarming,
    /// Julia has no immediate permits.
    JuliaAtCapacity,
    /// Waiting for Julia would miss the caller's deadline.
    DeadlineTooTight,
    /// Route, schema, manifest, or capability evidence is invalid.
    ContractInvalid,
    /// Benchmark evidence failed.
    BenchmarkFailed,
    /// Runtime error rate, warmup, or latency evidence is unstable.
    RuntimeUnstable,
    /// No Julia capacity is available.
    NoCapacity,
    /// Julia queue pressure is too high for immediate dispatch.
    QueuePressure,
    /// Predicted cost exceeds predicted Julia benefit.
    CostExceedsBenefit,
}

/// Inert scheduling plan for Julia compute work.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct JuliaSchedulePlan {
    /// Recommended action.
    pub action: JuliaScheduleAction,
    /// Reason for the action.
    pub reason: JuliaScheduleReason,
    /// Capability covered by this plan.
    pub capability: LaneCapability,
    /// Julia profile identifier covered by this plan.
    pub profile_id: String,
    /// Coarse readiness used by the plan.
    pub readiness: ReadinessState,
    /// Coarse pressure used by the plan.
    pub pressure: PressureLevel,
    /// Recommended batch size for this scheduling wave.
    pub selected_batch_size: u32,
    /// Recommended maximum in-flight count for this profile.
    pub max_in_flight_recommendation: u32,
    /// Whether owner-defined Rust fallback is available.
    pub fallback_available: bool,
    /// Predicted latency used by deadline decisions.
    pub predicted_latency_ms: u32,
    /// Predicted benefit score for Julia execution.
    pub benefit_score: i32,
    /// Predicted cost score for Julia execution.
    pub cost_score: i32,
    /// Benefit minus cost.
    pub confidence_score: i32,
    /// Optional batchability key forwarded from the task shape.
    pub batchability_key: Option<String>,
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
