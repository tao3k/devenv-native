//! Profile-aware Julia compute scheduling contracts.
//!
//! The scheduler translates owner-supplied readiness, runtime stats, and task
//! shape evidence into an inert plan. It does not call Julia, mutate queues,
//! execute Rust fallback code, or own any Wendao domain algorithm.

use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::{
    BenchmarkState, JuliaReadinessEvidence, LaneCapability, PressureLevel, ReadinessState,
    WarmupState,
};

/// Julia schedule profile identifier.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JuliaScheduleProfileId(String);

impl JuliaScheduleProfileId {
    /// Returns the profile identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the identifier into its owned string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for JuliaScheduleProfileId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for JuliaScheduleProfileId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl Deref for JuliaScheduleProfileId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl PartialEq<&str> for JuliaScheduleProfileId {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// Predicted Julia scheduling latency in milliseconds.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JuliaScheduleLatencyMs(u32);

impl JuliaScheduleLatencyMs {
    /// Returns the latency in milliseconds.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl From<u32> for JuliaScheduleLatencyMs {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// Key proving compatible Julia schedule requests can batch together.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JuliaScheduleBatchabilityKey(String);

impl JuliaScheduleBatchabilityKey {
    /// Returns the batchability key as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for JuliaScheduleBatchabilityKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Deref for JuliaScheduleBatchabilityKey {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

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
    pub(super) const fn benefit_bonus(self) -> i32 {
        match self {
            Self::Simple => 0,
            Self::Balanced => 40,
            Self::Heavy => 90,
        }
    }

    pub(super) const fn batch_cap(self) -> u32 {
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

    pub(super) fn normalized_rows(&self) -> u32 {
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
    pub profile_id: JuliaScheduleProfileId,
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
    pub predicted_latency_ms: JuliaScheduleLatencyMs,
    /// Predicted benefit score for Julia execution.
    pub benefit_score: i32,
    /// Predicted cost score for Julia execution.
    pub cost_score: i32,
    /// Benefit minus cost.
    pub confidence_score: i32,
    /// Optional batchability key forwarded from the task shape.
    pub batchability_key: Option<JuliaScheduleBatchabilityKey>,
}
