//! Worker-facing activity lifecycle helpers.

use crate::{
    ActivityCompletedJournalRecord, ActivityFailedJournalRecord, ActivityFailure,
    ActivityJournalScope, ActivityJournalWriteOutcome, ActivityResult, ControlLedger,
    ControlResult, ErrorCode, WorkerActivityTask, WorkerId, record_activity_completed_idempotent,
    record_activity_failed_idempotent, record_activity_started_idempotent,
};

/// Worker request for starting one durable activity task attempt.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkerActivityStartRecord {
    /// Worker-facing durable task envelope.
    pub task: WorkerActivityTask,
    /// Worker that starts the attempt.
    pub worker_id: WorkerId,
    /// Event timestamp supplied by caller.
    pub started_at_ms: u64,
}

impl WorkerActivityStartRecord {
    /// Creates a worker activity start request.
    #[must_use]
    pub const fn new(task: WorkerActivityTask, worker_id: WorkerId, started_at_ms: u64) -> Self {
        Self {
            task,
            worker_id,
            started_at_ms,
        }
    }
}

/// Worker request for completing one durable activity task.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkerActivityCompletedRecord {
    /// Worker-facing durable task envelope.
    pub task: WorkerActivityTask,
    /// Event timestamp supplied by caller.
    pub completed_at_ms: u64,
    /// Completion payload.
    pub result: ActivityResult,
}

impl WorkerActivityCompletedRecord {
    /// Creates a worker activity completion request.
    #[must_use]
    pub const fn new(
        task: WorkerActivityTask,
        completed_at_ms: u64,
        result: ActivityResult,
    ) -> Self {
        Self {
            task,
            completed_at_ms,
            result,
        }
    }
}

/// Worker request for failing one durable activity task attempt.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkerActivityFailedRecord {
    /// Worker-facing durable task envelope.
    pub task: WorkerActivityTask,
    /// Event timestamp supplied by caller.
    pub failed_at_ms: u64,
    /// Failure error code.
    pub error_code: ErrorCode,
    /// Failure message.
    pub message: String,
    /// Whether retry policy may schedule another attempt.
    pub retryable: bool,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Named input for constructing a worker activity failure record.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkerActivityFailureInput {
    /// Worker-facing durable task envelope.
    pub task: WorkerActivityTask,
    /// Event timestamp supplied by caller.
    pub failed_at_ms: u64,
    /// Failure error code.
    pub error_code: ErrorCode,
    /// Failure message.
    pub message: String,
    /// Whether retry policy may schedule another attempt.
    pub retryable: bool,
}

impl WorkerActivityFailureInput {
    /// Creates a named worker activity failure input.
    #[must_use]
    pub fn new(
        task: WorkerActivityTask,
        error_code: ErrorCode,
        message: impl Into<String>,
    ) -> Self {
        Self {
            task,
            failed_at_ms: 0,
            error_code,
            message: message.into(),
            retryable: false,
        }
    }

    /// Sets the failure timestamp.
    #[must_use]
    pub const fn with_failed_at_ms(mut self, failed_at_ms: u64) -> Self {
        self.failed_at_ms = failed_at_ms;
        self
    }

    /// Sets whether retry policy may schedule another attempt.
    #[must_use]
    pub const fn with_retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }
}

impl WorkerActivityFailedRecord {
    /// Creates a worker activity failure request.
    #[must_use]
    pub fn new(input: WorkerActivityFailureInput) -> Self {
        Self {
            task: input.task,
            failed_at_ms: input.failed_at_ms,
            error_code: input.error_code,
            message: input.message,
            retryable: input.retryable,
            metadata: serde_json::Value::Null,
        }
    }

    /// Sets extension metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Records a worker activity start with duplicate and transition guards.
///
/// # Errors
///
/// Returns a control error when the worker task cannot be started, replay
/// fails, or the ledger append fails.
pub fn record_worker_activity_started_idempotent<L>(
    ledger: &L,
    request: WorkerActivityStartRecord,
) -> ControlResult<ActivityJournalWriteOutcome>
where
    L: ControlLedger + ?Sized,
{
    let record = crate::ActivityStartedJournalRecord::new(
        scope_for_task(&request.task),
        request.started_at_ms,
        request.task.activity_id,
        request.task.next_attempt,
    )
    .with_worker_id(request.worker_id);
    record_activity_started_idempotent(ledger, record)
}

/// Records a worker activity completion with duplicate and transition guards.
///
/// # Errors
///
/// Returns a control error when the worker task cannot be completed, replay
/// fails, or the ledger append fails.
pub fn record_worker_activity_completed_idempotent<L>(
    ledger: &L,
    request: WorkerActivityCompletedRecord,
) -> ControlResult<ActivityJournalWriteOutcome>
where
    L: ControlLedger + ?Sized,
{
    let record = ActivityCompletedJournalRecord::new(
        scope_for_task(&request.task),
        request.completed_at_ms,
        request.task.activity_id,
        request.result,
    );
    record_activity_completed_idempotent(ledger, record)
}

/// Records a worker activity failure with duplicate and transition guards.
///
/// # Errors
///
/// Returns a control error when the worker task cannot be failed, replay fails,
/// or the ledger append fails.
pub fn record_worker_activity_failed_idempotent<L>(
    ledger: &L,
    request: WorkerActivityFailedRecord,
) -> ControlResult<ActivityJournalWriteOutcome>
where
    L: ControlLedger + ?Sized,
{
    let failure = ActivityFailure {
        error_code: request.error_code,
        message: request.message,
        retryable: request.retryable,
        attempt: request.task.next_attempt,
        metadata: request.metadata,
    };
    let record = ActivityFailedJournalRecord::new(
        scope_for_task(&request.task),
        request.failed_at_ms,
        request.task.activity_id,
        failure,
    );
    record_activity_failed_idempotent(ledger, record)
}

fn scope_for_task(task: &WorkerActivityTask) -> ActivityJournalScope {
    match &task.step_id {
        Some(step_id) => ActivityJournalScope::step(task.run_id.clone(), step_id.clone()),
        None => ActivityJournalScope::run(task.run_id.clone()),
    }
}
