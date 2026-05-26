//! Worker-facing activity lifecycle helpers.

use crate::{
    ActivityCompletedJournalRecord, ActivityFailedJournalRecord, ActivityFailure,
    ActivityJournalScope, ActivityJournalWriteOutcome, ActivityResult,
    ActivityStartedJournalRecord, ControlError, ControlLedger, ControlResult, ErrorCode,
    WorkerActivityTask, WorkerId, record_activity_completed_idempotent,
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

    /// Converts this worker-facing request into the generic activity journal
    /// record it will append.
    #[must_use]
    pub fn into_activity_started_record(self) -> ActivityStartedJournalRecord {
        let Self {
            task,
            worker_id,
            started_at_ms,
        } = self;
        let scope = scope_for_task(&task);
        ActivityStartedJournalRecord::new(scope, started_at_ms, task.activity_id, task.next_attempt)
            .with_worker_id(worker_id)
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

    /// Converts this worker-facing request into the generic activity journal
    /// record it will append.
    #[must_use]
    pub fn into_activity_completed_record(self) -> ActivityCompletedJournalRecord {
        let Self {
            task,
            completed_at_ms,
            result,
        } = self;
        let scope = scope_for_task(&task);
        ActivityCompletedJournalRecord::new(scope, completed_at_ms, task.activity_id, result)
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

    /// Creates a named worker activity failure input and validates the message.
    ///
    /// # Errors
    ///
    /// Returns a control error when the failure message is blank.
    pub fn try_new(
        task: WorkerActivityTask,
        error_code: ErrorCode,
        message: impl Into<String>,
    ) -> ControlResult<Self> {
        let message = message.into();
        validate_worker_failure_message(&message)?;
        Ok(Self::new(task, error_code, message))
    }

    /// Validates a worker failure diagnostic message.
    ///
    /// # Errors
    ///
    /// Returns a control error when the failure message is blank.
    pub fn validate_message(message: &str) -> ControlResult<()> {
        validate_worker_failure_message(message)
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

    /// Creates a worker activity failure request and validates the input.
    ///
    /// # Errors
    ///
    /// Returns a control error when the failure message is blank.
    pub fn try_new(input: WorkerActivityFailureInput) -> ControlResult<Self> {
        validate_worker_failure_message(&input.message)?;
        Ok(Self::new(input))
    }

    /// Sets extension metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Converts this worker-facing request into the generic activity journal
    /// record it will append.
    #[must_use]
    pub fn into_activity_failed_record(self) -> ActivityFailedJournalRecord {
        let Self {
            task,
            failed_at_ms,
            error_code,
            message,
            retryable,
            metadata,
        } = self;
        let scope = scope_for_task(&task);
        let failure = ActivityFailure {
            error_code,
            message,
            retryable,
            attempt: task.next_attempt,
            metadata,
        };
        ActivityFailedJournalRecord::new(scope, failed_at_ms, task.activity_id, failure)
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
    record_activity_started_idempotent(ledger, request.into_activity_started_record())
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
    record_activity_completed_idempotent(ledger, request.into_activity_completed_record())
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
    record_activity_failed_idempotent(ledger, request.into_activity_failed_record())
}

fn scope_for_task(task: &WorkerActivityTask) -> ActivityJournalScope {
    match &task.step_id {
        Some(step_id) => ActivityJournalScope::step(task.run_id.clone(), step_id.clone()),
        None => ActivityJournalScope::run(task.run_id.clone()),
    }
}

fn validate_worker_failure_message(message: &str) -> ControlResult<()> {
    if message.trim().is_empty() {
        return Err(ControlError::InvalidEventSequence {
            message: "worker activity failure message must not be blank".to_owned(),
        });
    }
    Ok(())
}
