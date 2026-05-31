//! Core control-plane data model.

use crate::{
    ActivityId, ActivityType, ArtifactId, ArtifactKind, ErrorCode, EvidenceId, GateName,
    IdempotencyKey, LeaseId, LlmModelId, RunId, SignalName, StepId, TaskQueue, TimerId, VersionKey,
    WorkerActivityTask, WorkerId,
};
use crate::{ControlError, ControlResult};

/// Run lifecycle status reconstructed from control events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    /// Intent captured but not yet admitted.
    #[default]
    Draft,
    /// Run admitted by the control planner.
    Admitted,
    /// Runnable work has been planned.
    Planned,
    /// At least one step is actively running.
    Running,
    /// Run is waiting on a tool, worker, or human boundary.
    Waiting,
    /// Recovery is in progress.
    Recovering,
    /// Run is blocked by a deterministic gate or missing prerequisite.
    Blocked,
    /// Run completed successfully.
    Completed,
    /// Run failed terminally.
    Failed,
    /// Run was intentionally aborted.
    Aborted,
}

/// Step lifecycle status reconstructed from control events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    /// Step declared but not yet queued.
    #[default]
    Pending,
    /// Step is waiting in a hot-state queue.
    Queued,
    /// Step has an active lease.
    Leased,
    /// Step execution has started.
    Running,
    /// Step is waiting on an external boundary.
    Waiting,
    /// Step is in recovery.
    Recovering,
    /// Step succeeded.
    Succeeded,
    /// Step failed.
    Failed,
    /// Step is blocked by a deterministic condition.
    Blocked,
    /// Step was cancelled.
    Cancelled,
}

/// Why a run or step is waiting.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaitReason {
    /// Waiting on tool output.
    Tool,
    /// Waiting on human approval or input.
    Human,
    /// Waiting on worker capacity.
    Worker,
    /// Waiting on external IO or service response.
    External,
}

/// Optional budget for one run or step.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Budget {
    /// Maximum wall time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall_time_ms: Option<u64>,
    /// Maximum model or provider tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens: Option<u64>,
    /// Maximum cost in USD micros.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_usd_micros: Option<u64>,
}

/// One observed cost row.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CostObservation {
    /// Provider or tool name.
    pub provider: String,
    /// Optional model id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Prompt tokens.
    #[serde(default)]
    pub prompt_tokens: u64,
    /// Completion tokens.
    #[serde(default)]
    pub completion_tokens: u64,
    /// Total tokens when provider reports a direct value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u64>,
    /// Cost in USD micros.
    #[serde(default)]
    pub cost_usd_micros: u64,
    /// Provider/tool latency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}

impl CostObservation {
    /// Returns total tokens, deriving the value when needed.
    #[must_use]
    pub fn observed_total_tokens(&self) -> u64 {
        self.total_tokens
            .unwrap_or(self.prompt_tokens + self.completion_tokens)
    }
}

/// Declarative retry policy for one activity task.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ActivityRetryPolicy {
    /// Maximum attempts, including the first attempt.
    pub max_attempts: u32,
    /// Initial retry interval.
    #[serde(default)]
    pub initial_interval_ms: u64,
    /// Maximum retry interval.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_interval_ms: Option<u64>,
    /// Backoff multiplier in thousandths. `2000` means `2.0x`.
    #[serde(default)]
    pub backoff_multiplier_millis: u32,
    /// Error codes that must not be retried.
    #[serde(default)]
    pub non_retryable_error_codes: Vec<ErrorCode>,
}

impl ActivityRetryPolicy {
    /// Default exponential backoff multiplier in thousandths.
    pub const DEFAULT_BACKOFF_MULTIPLIER_MILLIS: u32 = 2_000;

    /// Creates a retry policy with deterministic defaults.
    ///
    /// # Errors
    ///
    /// Returns a control error when `max_attempts` is zero.
    pub fn new(max_attempts: u32) -> ControlResult<Self> {
        let policy = Self {
            max_attempts,
            initial_interval_ms: 0,
            max_interval_ms: None,
            backoff_multiplier_millis: Self::DEFAULT_BACKOFF_MULTIPLIER_MILLIS,
            non_retryable_error_codes: Vec::new(),
        };
        policy.validate()?;
        Ok(policy)
    }

    /// Sets the initial retry interval.
    #[must_use]
    pub const fn with_initial_interval_ms(mut self, initial_interval_ms: u64) -> Self {
        self.initial_interval_ms = initial_interval_ms;
        self
    }

    /// Sets the maximum retry interval.
    #[must_use]
    pub const fn with_max_interval_ms(mut self, max_interval_ms: u64) -> Self {
        self.max_interval_ms = Some(max_interval_ms);
        self
    }

    /// Sets the backoff multiplier in thousandths.
    ///
    /// # Errors
    ///
    /// Returns a control error when the multiplier is zero.
    pub fn with_backoff_multiplier_millis(
        mut self,
        backoff_multiplier_millis: u32,
    ) -> ControlResult<Self> {
        self.backoff_multiplier_millis = backoff_multiplier_millis;
        self.validate()?;
        Ok(self)
    }

    /// Adds one non-retryable error code.
    #[must_use]
    pub fn with_non_retryable_error_code(mut self, error_code: ErrorCode) -> Self {
        if !self
            .non_retryable_error_codes
            .iter()
            .any(|existing| existing == &error_code)
        {
            self.non_retryable_error_codes.push(error_code);
        }
        self
    }

    /// Validates this retry policy.
    ///
    /// # Errors
    ///
    /// Returns a control error when attempts, backoff, or interval bounds are
    /// invalid.
    pub fn validate(&self) -> ControlResult<()> {
        if self.max_attempts == 0 {
            return Err(invalid_activity_policy(
                "activity retry policy requires max_attempts to be at least 1",
            ));
        }
        if self.backoff_multiplier_millis == 0 {
            return Err(invalid_activity_policy(
                "activity retry policy requires non-zero backoff_multiplier_millis",
            ));
        }
        if let Some(max_interval_ms) = self.max_interval_ms
            && max_interval_ms < self.initial_interval_ms
        {
            return Err(invalid_activity_policy(
                "activity retry policy max_interval_ms cannot be lower than initial_interval_ms",
            ));
        }
        Ok(())
    }

    /// Decides whether a failed activity may schedule another attempt.
    ///
    /// # Errors
    ///
    /// Returns a control error when the policy is invalid or the failed
    /// attempt number is zero.
    pub fn decide_after_failure(
        &self,
        failure: &ActivityFailure,
    ) -> ControlResult<ActivityRetryDecision> {
        self.validate()?;
        if failure.attempt == 0 {
            return Err(invalid_activity_policy(
                "activity failure attempt must be at least 1",
            ));
        }
        if !failure.retryable {
            return Ok(ActivityRetryDecision::DoNotRetry {
                reason: ActivityRetryStopReason::FailureMarkedNonRetryable,
            });
        }
        if self
            .non_retryable_error_codes
            .iter()
            .any(|error_code| error_code == &failure.error_code)
        {
            return Ok(ActivityRetryDecision::DoNotRetry {
                reason: ActivityRetryStopReason::NonRetryableErrorCode,
            });
        }
        if failure.attempt >= self.max_attempts {
            return Ok(ActivityRetryDecision::DoNotRetry {
                reason: ActivityRetryStopReason::AttemptsExhausted,
            });
        }

        Ok(ActivityRetryDecision::Retry {
            next_attempt: failure.attempt + 1,
            backoff_ms: self.retry_backoff_ms_after_failed_attempt(failure.attempt)?,
        })
    }

    /// Returns the backoff after the supplied failed attempt.
    ///
    /// # Errors
    ///
    /// Returns a control error when the policy is invalid or `failed_attempt`
    /// is zero.
    pub fn retry_backoff_ms_after_failed_attempt(&self, failed_attempt: u32) -> ControlResult<u64> {
        self.validate()?;
        if failed_attempt == 0 {
            return Err(invalid_activity_policy(
                "activity failed_attempt must be at least 1",
            ));
        }

        let mut backoff = u128::from(self.initial_interval_ms);
        for _ in 1..failed_attempt {
            backoff = backoff.saturating_mul(u128::from(self.backoff_multiplier_millis)) / 1_000;
        }

        let capped = self
            .max_interval_ms
            .map_or(backoff, |max_interval_ms| {
                backoff.min(u128::from(max_interval_ms))
            })
            .min(u128::from(u64::MAX));
        Ok(u64::try_from(capped).unwrap_or(u64::MAX))
    }
}

/// Deterministic retry decision after an activity failure.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityRetryDecision {
    /// Another attempt may be scheduled.
    Retry {
        /// Attempt number to schedule next.
        next_attempt: u32,
        /// Backoff to wait before the next attempt.
        backoff_ms: u64,
    },
    /// No further attempt may be scheduled.
    DoNotRetry {
        /// Deterministic stop reason.
        reason: ActivityRetryStopReason,
    },
}

/// Reason a retry decision stopped instead of scheduling another attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityRetryStopReason {
    /// The activity failure was explicitly marked non-retryable.
    FailureMarkedNonRetryable,
    /// The failure error code is listed as non-retryable by policy.
    NonRetryableErrorCode,
    /// The policy's maximum attempt count has already been reached.
    AttemptsExhausted,
}

/// Workflow-neutral activity task scheduled by the control plane.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivityTask {
    /// Stable activity id within the owning run.
    pub activity_id: ActivityId,
    /// Logical activity type, such as `llm.plan` or `wendao.search`.
    pub activity_type: ActivityType,
    /// Typed task queue or dispatch lane.
    pub task_queue: TaskQueue,
    /// Optional claim-check input reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_ref: Option<ArtifactRef>,
    /// Idempotency key supplied by the scheduler.
    pub idempotency_key: IdempotencyKey,
    /// Optional retry policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_policy: Option<ActivityRetryPolicy>,
    /// Optional execution timeout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl ActivityTask {
    /// Creates an activity task with deterministic defaults.
    #[must_use]
    pub fn new(
        activity_id: ActivityId,
        activity_type: ActivityType,
        task_queue: TaskQueue,
        idempotency_key: IdempotencyKey,
    ) -> Self {
        Self {
            activity_id,
            activity_type,
            task_queue,
            input_ref: None,
            idempotency_key,
            retry_policy: None,
            timeout_ms: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Sets the claim-check input reference.
    #[must_use]
    pub fn with_input_ref(mut self, input_ref: ArtifactRef) -> Self {
        self.input_ref = Some(input_ref);
        self
    }

    /// Sets the retry policy.
    #[must_use]
    pub fn with_retry_policy(mut self, retry_policy: ActivityRetryPolicy) -> Self {
        self.retry_policy = Some(retry_policy);
        self
    }

    /// Sets the execution timeout.
    #[must_use]
    pub const fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// Validates this activity task's optional policy fields.
    ///
    /// # Errors
    ///
    /// Returns a control error when the retry policy or timeout is invalid.
    pub fn validate(&self) -> ControlResult<()> {
        if let Some(retry_policy) = &self.retry_policy {
            retry_policy.validate()?;
        }
        if matches!(self.timeout_ms, Some(0)) {
            return Err(invalid_activity_policy(
                "activity task timeout_ms must be non-zero when supplied",
            ));
        }
        Ok(())
    }
}

fn invalid_activity_policy(message: &str) -> ControlError {
    ControlError::InvalidEventSequence {
        message: message.to_owned(),
    }
}

/// LLM activity payload governed by the control plane.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LlmActivityRequest {
    /// Model id requested by the controller.
    pub model: LlmModelId,
    /// Claim-check prompt reference.
    pub prompt_ref: ArtifactRef,
    /// Optional claim-check context reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_ref: Option<ArtifactRef>,
    /// Optional hash of the tool schema visible to the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_schema_hash: Option<String>,
    /// Temperature encoded in thousandths for stable serialization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature_millis: Option<u32>,
    /// Maximum completion tokens requested.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Optional claim-check response schema reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_schema_ref: Option<ArtifactRef>,
    /// Optional budget applied to this model call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget: Option<Budget>,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl LlmActivityRequest {
    /// Creates an LLM activity payload with deterministic defaults.
    #[must_use]
    pub fn new(model: LlmModelId, prompt_ref: ArtifactRef) -> Self {
        Self {
            model,
            prompt_ref,
            context_ref: None,
            tool_schema_hash: None,
            temperature_millis: None,
            max_tokens: None,
            response_schema_ref: None,
            budget: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Sets the claim-check context reference.
    #[must_use]
    pub fn with_context_ref(mut self, context_ref: ArtifactRef) -> Self {
        self.context_ref = Some(context_ref);
        self
    }

    /// Sets the visible tool-schema hash.
    #[must_use]
    pub fn with_tool_schema_hash(mut self, tool_schema_hash: impl Into<String>) -> Self {
        self.tool_schema_hash = Some(tool_schema_hash.into());
        self
    }

    /// Sets the temperature in thousandths.
    #[must_use]
    pub const fn with_temperature_millis(mut self, temperature_millis: u32) -> Self {
        self.temperature_millis = Some(temperature_millis);
        self
    }

    /// Sets the maximum completion tokens.
    #[must_use]
    pub const fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Sets the claim-check response schema reference.
    #[must_use]
    pub fn with_response_schema_ref(mut self, response_schema_ref: ArtifactRef) -> Self {
        self.response_schema_ref = Some(response_schema_ref);
        self
    }

    /// Sets the LLM activity budget.
    #[must_use]
    pub fn with_budget(mut self, budget: Budget) -> Self {
        self.budget = Some(budget);
        self
    }

    /// Validates the LLM activity payload.
    ///
    /// # Errors
    ///
    /// Returns a control error when token or hash fields are invalid.
    pub fn validate(&self) -> ControlResult<()> {
        if matches!(self.max_tokens, Some(0)) {
            return Err(invalid_llm_activity_contract(
                "llm activity max_tokens must be non-zero when supplied",
            ));
        }
        if self
            .tool_schema_hash
            .as_ref()
            .is_some_and(|hash| hash.trim().is_empty())
        {
            return Err(invalid_llm_activity_contract(
                "llm activity tool_schema_hash must not be blank when supplied",
            ));
        }
        Ok(())
    }
}

/// Activity task plus LLM payload.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LlmActivityTask {
    /// Generic activity task envelope.
    pub task: ActivityTask,
    /// LLM-specific request payload.
    pub request: LlmActivityRequest,
}

impl LlmActivityTask {
    /// Creates an LLM activity task binding.
    #[must_use]
    pub const fn new(task: ActivityTask, request: LlmActivityRequest) -> Self {
        Self { task, request }
    }

    /// Validates the LLM activity binding.
    ///
    /// # Errors
    ///
    /// Returns a control error when the task, request, activity type, or task
    /// queue is invalid for an LLM activity.
    pub fn validate(&self) -> ControlResult<()> {
        self.task.validate()?;
        self.request.validate()?;
        if !self.task.activity_type.as_str().starts_with("llm.") {
            return Err(invalid_llm_activity_contract(
                "llm activity task requires an llm.* activity_type",
            ));
        }
        if !self.task.task_queue.as_str().starts_with("llm.") {
            return Err(invalid_llm_activity_contract(
                "llm activity task requires an llm.* task_queue",
            ));
        }
        Ok(())
    }
}

fn invalid_llm_activity_contract(message: &str) -> ControlError {
    ControlError::InvalidEventSequence {
        message: message.to_owned(),
    }
}

/// Activity completion payload.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivityResult {
    /// Optional claim-check output reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_ref: Option<ArtifactRef>,
    /// Optional output content hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_hash: Option<String>,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Activity failure payload.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivityFailure {
    /// Stable error code.
    pub error_code: ErrorCode,
    /// Human-readable diagnostic.
    pub message: String,
    /// Whether the activity may be retried.
    pub retryable: bool,
    /// Attempt number, starting at one.
    pub attempt: u32,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// External signal recorded into the execution journal.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SignalRecord {
    /// Stable signal name.
    pub signal_name: SignalName,
    /// Optional claim-check payload reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_ref: Option<ArtifactRef>,
    /// Optional payload hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_hash: Option<String>,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Durable timer scheduled by a run or step.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TimerRecord {
    /// Stable timer id.
    pub timer_id: TimerId,
    /// Wall-clock fire time in Unix milliseconds.
    pub fire_at_ms: u64,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Pinned deterministic version or schema fact for replay.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VersionPin {
    /// Version key, such as `flowhub_version` or `tool_schema_hash`.
    pub version_key: VersionKey,
    /// Version value.
    pub version: String,
    /// Optional content hash associated with the pinned version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Evidence attached to a step.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EvidenceRef {
    /// Evidence id.
    pub evidence_id: EvidenceId,
    /// Optional required-evidence key this ref satisfies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requirement_key: Option<String>,
    /// Human-readable source.
    pub source: String,
    /// Optional URI, file path, or artifact route.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// Short summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Artifact attached to a run or step.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ArtifactRef {
    /// Artifact id.
    pub artifact_id: ArtifactId,
    /// Logical artifact kind.
    pub artifact_kind: ArtifactKind,
    /// URI, file path, or external route.
    pub uri: String,
    /// Optional content digest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_digest: Option<String>,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Gate evaluation result.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GateResult {
    /// Stable gate name.
    pub gate_name: GateName,
    /// Whether the gate passed.
    pub passed: bool,
    /// Whether required evidence coverage is complete.
    pub required_evidence_covered: bool,
    /// Required evidence keys covered by selected evidence.
    #[serde(default)]
    pub selected_required_evidence: Vec<String>,
    /// Required evidence keys still missing.
    #[serde(default)]
    pub missing_required_evidence: Vec<String>,
    /// Human-readable reasons.
    #[serde(default)]
    pub reasons: Vec<String>,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Recovery policy declared for a run or step.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecoveryPolicy {
    /// Maximum attempts.
    pub max_attempts: u32,
    /// Backoff before retry.
    #[serde(default)]
    pub backoff_ms: u64,
    /// Whether human approval is required before retry.
    #[serde(default)]
    pub require_human_approval: bool,
}

/// Runnable step entry for hot-state queues.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RunnableStep {
    /// Owning run id.
    pub run_id: RunId,
    /// Step id.
    pub step_id: StepId,
    /// Higher values are acquired first.
    #[serde(default)]
    pub priority: i64,
    /// Earliest acquisition time.
    #[serde(default)]
    pub not_before_ms: u64,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Runnable activity task entry for hot-state queues.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RunnableActivityTask {
    /// Worker-facing durable task envelope.
    pub task: WorkerActivityTask,
    /// Higher values are acquired first.
    #[serde(default)]
    pub priority: i64,
    /// Earliest acquisition time.
    #[serde(default)]
    pub not_before_ms: u64,
    /// Extension metadata for hot-state delivery.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Request for claiming an activity task within one durable run.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RunScopedActivityTaskClaimRequest {
    /// Worker requesting the lease.
    pub worker: WorkerRef,
    /// Owning run id.
    pub run_id: RunId,
    /// Optional task queue filter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_queue: Option<TaskQueue>,
    /// Claim timestamp.
    pub now_ms: u64,
    /// Lease TTL.
    pub lease_ttl_ms: u64,
}

impl RunScopedActivityTaskClaimRequest {
    /// Creates a run-scoped activity task claim request.
    #[must_use]
    pub const fn new(worker: WorkerRef, run_id: RunId, now_ms: u64, lease_ttl_ms: u64) -> Self {
        Self {
            worker,
            run_id,
            task_queue: None,
            now_ms,
            lease_ttl_ms,
        }
    }

    /// Filters the claim to one task queue.
    #[must_use]
    pub fn with_task_queue(mut self, task_queue: TaskQueue) -> Self {
        self.task_queue = Some(task_queue);
        self
    }
}

/// Worker requesting hot-state leases.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkerRef {
    /// Worker id.
    pub worker_id: WorkerId,
    /// Worker capabilities.
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Active step lease.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StepLease {
    /// Lease id.
    pub lease_id: LeaseId,
    /// Owning run id.
    pub run_id: RunId,
    /// Leased step id.
    pub step_id: StepId,
    /// Owning worker id.
    pub worker_id: WorkerId,
    /// Acquisition timestamp.
    pub acquired_at_ms: u64,
    /// Expiry timestamp.
    pub expires_at_ms: u64,
}

impl StepLease {
    /// Returns true when the lease is still valid at `now_ms`.
    #[must_use]
    pub const fn is_active_at(&self, now_ms: u64) -> bool {
        now_ms < self.expires_at_ms
    }
}

/// Active activity-task lease.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ActivityTaskLease {
    /// Lease id.
    pub lease_id: LeaseId,
    /// Owning run id.
    pub run_id: RunId,
    /// Owning step id when the activity is step-scoped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<StepId>,
    /// Leased activity id.
    pub activity_id: ActivityId,
    /// Owning worker id.
    pub worker_id: WorkerId,
    /// Acquisition timestamp.
    pub acquired_at_ms: u64,
    /// Expiry timestamp.
    pub expires_at_ms: u64,
}

impl ActivityTaskLease {
    /// Returns true when the lease is still valid at `now_ms`.
    #[must_use]
    pub const fn is_active_at(&self, now_ms: u64) -> bool {
        now_ms < self.expires_at_ms
    }
}

/// Worker heartbeat stored in hot state.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkerHeartbeat {
    /// Worker id.
    pub worker_id: WorkerId,
    /// Last heartbeat timestamp.
    pub observed_at_ms: u64,
    /// Heartbeat expiry timestamp.
    pub expires_at_ms: u64,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Read-only snapshot of hot scheduling state.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HotStateSnapshot {
    /// Timestamp used by the caller to interpret expiry state.
    pub observed_at_ms: u64,
    /// Steps currently waiting in the runnable queue.
    #[serde(default)]
    pub pending_steps: Vec<RunnableStep>,
    /// Steps currently protected by active or expired leases.
    #[serde(default)]
    pub leased_steps: Vec<HotStateLeasedStep>,
    /// Activity tasks currently waiting in the hot activity queue.
    #[serde(default)]
    pub pending_activity_tasks: Vec<RunnableActivityTask>,
    /// Activity tasks currently protected by active or expired leases.
    #[serde(default)]
    pub leased_activity_tasks: Vec<HotStateLeasedActivityTask>,
    /// Worker heartbeat payloads still visible in hot state.
    #[serde(default)]
    pub worker_heartbeats: Vec<WorkerHeartbeat>,
}

impl HotStateSnapshot {
    /// Creates an empty snapshot observed at `observed_at_ms`.
    #[must_use]
    pub const fn new(observed_at_ms: u64) -> Self {
        Self {
            observed_at_ms,
            pending_steps: Vec::new(),
            leased_steps: Vec::new(),
            pending_activity_tasks: Vec::new(),
            leased_activity_tasks: Vec::new(),
            worker_heartbeats: Vec::new(),
        }
    }

    /// Returns active lease count at the snapshot timestamp.
    #[must_use]
    pub fn active_lease_count(&self) -> usize {
        self.leased_steps
            .iter()
            .filter(|leased| leased.lease.is_active_at(self.observed_at_ms))
            .count()
    }

    /// Returns expired lease count at the snapshot timestamp.
    #[must_use]
    pub fn expired_lease_count(&self) -> usize {
        self.leased_steps.len() - self.active_lease_count()
    }

    /// Returns active activity-task lease count at the snapshot timestamp.
    #[must_use]
    pub fn active_activity_task_lease_count(&self) -> usize {
        self.leased_activity_tasks
            .iter()
            .filter(|leased| leased.lease.is_active_at(self.observed_at_ms))
            .count()
    }

    /// Returns expired activity-task lease count at the snapshot timestamp.
    #[must_use]
    pub fn expired_activity_task_lease_count(&self) -> usize {
        self.leased_activity_tasks.len() - self.active_activity_task_lease_count()
    }

    /// Returns live heartbeat count at the snapshot timestamp.
    #[must_use]
    pub fn live_heartbeat_count(&self) -> usize {
        self.worker_heartbeats
            .iter()
            .filter(|heartbeat| self.observed_at_ms < heartbeat.expires_at_ms)
            .count()
    }
}

/// One leased hot-state step plus its original runnable payload.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HotStateLeasedStep {
    /// Original runnable step payload.
    pub step: RunnableStep,
    /// Current lease payload.
    pub lease: StepLease,
}

/// One leased hot-state activity task plus its original runnable payload.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HotStateLeasedActivityTask {
    /// Original runnable activity task payload.
    pub activity_task: RunnableActivityTask,
    /// Current lease payload.
    pub lease: ActivityTaskLease,
}
