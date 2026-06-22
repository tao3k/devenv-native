//! Activity journal data model.

use super::metadata::llm_activity_schedule_task;
use crate::{
    ActivityFailure, ActivityId, ActivityResult, ActivityTask, ControlEvent, ControlEventKind,
    ControlEventRecord, LlmActivityAdmission, RunId, StepId, ToolActivityAdmission, WorkerId,
};

/// Named request for recording one admitted activity scheduling fact.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AdmittedActivityScheduleRecord {
    /// Owning run id.
    pub run_id: RunId,
    /// Optional owning step id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<StepId>,
    /// Event timestamp supplied by caller.
    pub occurred_at_ms: u64,
    /// Already admitted tool activity.
    pub admission: ToolActivityAdmission,
}

/// Named request for recording one admitted LLM activity scheduling fact.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AdmittedLlmActivityScheduleRecord {
    /// Owning run id.
    pub run_id: RunId,
    /// Optional owning step id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<StepId>,
    /// Event timestamp supplied by caller.
    pub occurred_at_ms: u64,
    /// Already admitted LLM activity.
    pub admission: LlmActivityAdmission,
}

/// Named request for recording one already admitted generic activity task.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AdmittedActivityTaskScheduleRecord {
    /// Owning run id.
    pub run_id: RunId,
    /// Optional owning step id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<StepId>,
    /// Event timestamp supplied by caller.
    pub occurred_at_ms: u64,
    /// Already admitted workflow-neutral activity task.
    pub task: ActivityTask,
}

impl AdmittedActivityScheduleRecord {
    /// Creates a run-scoped admitted activity schedule record request.
    #[must_use]
    pub const fn run(run_id: RunId, occurred_at_ms: u64, admission: ToolActivityAdmission) -> Self {
        Self {
            run_id,
            step_id: None,
            occurred_at_ms,
            admission,
        }
    }

    /// Creates a step-scoped admitted activity schedule record request.
    #[must_use]
    pub const fn step(
        run_id: RunId,
        step_id: StepId,
        occurred_at_ms: u64,
        admission: ToolActivityAdmission,
    ) -> Self {
        Self {
            run_id,
            step_id: Some(step_id),
            occurred_at_ms,
            admission,
        }
    }

    /// Converts this record into its replayable control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        let Self {
            run_id,
            step_id,
            occurred_at_ms,
            admission,
        } = self;
        let event_kind = ControlEventKind::ActivityScheduled {
            task: admission.task,
        };
        match step_id {
            Some(step_id) => ControlEvent::step(run_id, step_id, occurred_at_ms, event_kind),
            None => ControlEvent::run(run_id, occurred_at_ms, event_kind),
        }
    }
}

impl AdmittedLlmActivityScheduleRecord {
    /// Creates a run-scoped admitted LLM activity schedule record request.
    #[must_use]
    pub const fn run(run_id: RunId, occurred_at_ms: u64, admission: LlmActivityAdmission) -> Self {
        Self {
            run_id,
            step_id: None,
            occurred_at_ms,
            admission,
        }
    }

    /// Creates a step-scoped admitted LLM activity schedule record request.
    #[must_use]
    pub const fn step(
        run_id: RunId,
        step_id: StepId,
        occurred_at_ms: u64,
        admission: LlmActivityAdmission,
    ) -> Self {
        Self {
            run_id,
            step_id: Some(step_id),
            occurred_at_ms,
            admission,
        }
    }

    /// Converts this record into its replayable control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        let Self {
            run_id,
            step_id,
            occurred_at_ms,
            admission,
        } = self;
        let event_kind = ControlEventKind::ActivityScheduled {
            task: llm_activity_schedule_task(&admission),
        };
        match step_id {
            Some(step_id) => ControlEvent::step(run_id, step_id, occurred_at_ms, event_kind),
            None => ControlEvent::run(run_id, occurred_at_ms, event_kind),
        }
    }
}

impl AdmittedActivityTaskScheduleRecord {
    /// Creates a run-scoped admitted activity task schedule record request.
    #[must_use]
    pub const fn run(run_id: RunId, occurred_at_ms: u64, task: ActivityTask) -> Self {
        Self {
            run_id,
            step_id: None,
            occurred_at_ms,
            task,
        }
    }

    /// Creates a step-scoped admitted activity task schedule record request.
    #[must_use]
    pub const fn step(
        run_id: RunId,
        step_id: StepId,
        occurred_at_ms: u64,
        task: ActivityTask,
    ) -> Self {
        Self {
            run_id,
            step_id: Some(step_id),
            occurred_at_ms,
            task,
        }
    }

    /// Converts this record into its replayable control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        let Self {
            run_id,
            step_id,
            occurred_at_ms,
            task,
        } = self;
        let event_kind = ControlEventKind::ActivityScheduled { task };
        match step_id {
            Some(step_id) => ControlEvent::step(run_id, step_id, occurred_at_ms, event_kind),
            None => ControlEvent::run(run_id, occurred_at_ms, event_kind),
        }
    }
}

/// Idempotent activity journal write status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityJournalWriteStatus {
    /// The helper appended a new event.
    Appended,
    /// An exact matching event already existed in durable history.
    AlreadyRecorded,
}

/// Result of a checked/idempotent activity journal write.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivityJournalWriteOutcome {
    /// Whether the helper appended a new event or found an existing one.
    pub status: ActivityJournalWriteStatus,
    /// The stored event record.
    pub record: ControlEventRecord,
}

impl ActivityJournalWriteOutcome {
    pub(super) fn appended(record: ControlEventRecord) -> Self {
        Self {
            status: ActivityJournalWriteStatus::Appended,
            record,
        }
    }

    pub(super) fn already_recorded(record: ControlEventRecord) -> Self {
        Self {
            status: ActivityJournalWriteStatus::AlreadyRecorded,
            record,
        }
    }
}

/// Journal scope for activity lifecycle facts.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "scope", rename_all = "snake_case")]
pub enum ActivityJournalScope {
    /// Record the activity fact at run scope.
    Run {
        /// Owning run id.
        run_id: RunId,
    },
    /// Record the activity fact at step scope.
    Step {
        /// Owning run id.
        run_id: RunId,
        /// Owning step id.
        step_id: StepId,
    },
}

impl ActivityJournalScope {
    /// Creates a run-scoped activity journal scope.
    #[must_use]
    pub const fn run(run_id: RunId) -> Self {
        Self::Run { run_id }
    }

    /// Creates a step-scoped activity journal scope.
    #[must_use]
    pub const fn step(run_id: RunId, step_id: StepId) -> Self {
        Self::Step { run_id, step_id }
    }

    fn into_event(self, occurred_at_ms: u64, kind: ControlEventKind) -> ControlEvent {
        match self {
            Self::Run { run_id } => ControlEvent::run(run_id, occurred_at_ms, kind),
            Self::Step { run_id, step_id } => {
                ControlEvent::step(run_id, step_id, occurred_at_ms, kind)
            }
        }
    }

    pub(super) fn run_id(&self) -> &RunId {
        match self {
            Self::Run { run_id } | Self::Step { run_id, .. } => run_id,
        }
    }

    pub(super) fn step_id(&self) -> Option<&StepId> {
        match self {
            Self::Run { .. } => None,
            Self::Step { step_id, .. } => Some(step_id),
        }
    }
}

/// Named request for recording one activity start fact.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivityStartedJournalRecord {
    /// Journal scope.
    pub scope: ActivityJournalScope,
    /// Event timestamp supplied by caller.
    pub occurred_at_ms: u64,
    /// Activity id.
    pub activity_id: ActivityId,
    /// Worker that started the attempt, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worker_id: Option<WorkerId>,
    /// Attempt number, starting at one.
    pub attempt: u32,
}

impl ActivityStartedJournalRecord {
    /// Creates an activity start record request.
    #[must_use]
    pub const fn new(
        scope: ActivityJournalScope,
        occurred_at_ms: u64,
        activity_id: ActivityId,
        attempt: u32,
    ) -> Self {
        Self {
            scope,
            occurred_at_ms,
            activity_id,
            worker_id: None,
            attempt,
        }
    }

    /// Sets the worker id that started this attempt.
    #[must_use]
    pub fn with_worker_id(mut self, worker_id: WorkerId) -> Self {
        self.worker_id = Some(worker_id);
        self
    }

    /// Converts this record into its replayable control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        let Self {
            scope,
            occurred_at_ms,
            activity_id,
            worker_id,
            attempt,
        } = self;
        scope.into_event(
            occurred_at_ms,
            ControlEventKind::ActivityStarted {
                activity_id,
                worker_id,
                attempt,
            },
        )
    }
}

/// Named request for recording one activity completion fact.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivityCompletedJournalRecord {
    /// Journal scope.
    pub scope: ActivityJournalScope,
    /// Event timestamp supplied by caller.
    pub occurred_at_ms: u64,
    /// Activity id.
    pub activity_id: ActivityId,
    /// Completion payload.
    pub result: ActivityResult,
}

impl ActivityCompletedJournalRecord {
    /// Creates an activity completion record request.
    #[must_use]
    pub const fn new(
        scope: ActivityJournalScope,
        occurred_at_ms: u64,
        activity_id: ActivityId,
        result: ActivityResult,
    ) -> Self {
        Self {
            scope,
            occurred_at_ms,
            activity_id,
            result,
        }
    }

    /// Converts this record into its replayable control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        let Self {
            scope,
            occurred_at_ms,
            activity_id,
            result,
        } = self;
        scope.into_event(
            occurred_at_ms,
            ControlEventKind::ActivityCompleted {
                activity_id,
                result,
            },
        )
    }
}

/// Named request for recording one activity failure fact.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivityFailedJournalRecord {
    /// Journal scope.
    pub scope: ActivityJournalScope,
    /// Event timestamp supplied by caller.
    pub occurred_at_ms: u64,
    /// Activity id.
    pub activity_id: ActivityId,
    /// Failure payload.
    pub failure: ActivityFailure,
}

impl ActivityFailedJournalRecord {
    /// Creates an activity failure record request.
    #[must_use]
    pub const fn new(
        scope: ActivityJournalScope,
        occurred_at_ms: u64,
        activity_id: ActivityId,
        failure: ActivityFailure,
    ) -> Self {
        Self {
            scope,
            occurred_at_ms,
            activity_id,
            failure,
        }
    }

    /// Converts this record into its replayable control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        let Self {
            scope,
            occurred_at_ms,
            activity_id,
            failure,
        } = self;
        scope.into_event(
            occurred_at_ms,
            ControlEventKind::ActivityFailed {
                activity_id,
                failure,
            },
        )
    }
}
