//! Durable run journal recording helpers.

use crate::{
    Budget, ControlEvent, ControlEventKind, ControlEventRecord, ControlLedger, ControlResult, RunId,
};

/// Named request for recording one created run fact.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RunCreatedJournalRecord {
    /// Owning run id.
    pub run_id: RunId,
    /// Human-readable run intent.
    pub intent: String,
    /// Optional run budget.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget: Option<Budget>,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
    /// Event timestamp supplied by caller or external input.
    pub occurred_at_ms: u64,
}

impl RunCreatedJournalRecord {
    /// Creates a run-created journal record request with no budget or metadata.
    #[must_use]
    pub fn new(run_id: RunId, intent: impl Into<String>, occurred_at_ms: u64) -> Self {
        Self {
            run_id,
            intent: intent.into(),
            budget: None,
            metadata: serde_json::Value::Null,
            occurred_at_ms,
        }
    }

    /// Sets the optional run budget.
    #[must_use]
    pub fn with_budget(mut self, budget: Budget) -> Self {
        self.budget = Some(budget);
        self
    }

    /// Sets extension metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Converts this request into the corresponding control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        let Self {
            run_id,
            intent,
            budget,
            metadata,
            occurred_at_ms,
        } = self;
        ControlEvent::run(
            run_id,
            occurred_at_ms,
            ControlEventKind::RunCreated {
                intent,
                budget,
                metadata,
            },
        )
    }
}

/// Records one run-created fact as an append-only control event.
///
/// # Errors
///
/// Returns a control error when the ledger append fails.
pub fn record_run_created<L>(
    ledger: &L,
    request: RunCreatedJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    ledger.append_event(request.into_event())
}

/// Named request for recording one run-admitted fact.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RunAdmittedJournalRecord {
    /// Owning run id.
    pub run_id: RunId,
    /// Event timestamp supplied by caller or external input.
    pub occurred_at_ms: u64,
}

impl RunAdmittedJournalRecord {
    /// Creates a run-admitted journal record request.
    #[must_use]
    pub const fn new(run_id: RunId, occurred_at_ms: u64) -> Self {
        Self {
            run_id,
            occurred_at_ms,
        }
    }

    /// Converts this request into the corresponding control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        ControlEvent::run(
            self.run_id,
            self.occurred_at_ms,
            ControlEventKind::RunAdmitted,
        )
    }
}

/// Records one run-admitted fact as an append-only control event.
///
/// # Errors
///
/// Returns a control error when the ledger append fails.
pub fn record_run_admitted<L>(
    ledger: &L,
    request: RunAdmittedJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    ledger.append_event(request.into_event())
}

/// Named request for recording one plan summary fact.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RunPlanRecordedJournalRecord {
    /// Owning run id.
    pub run_id: RunId,
    /// Human-readable plan summary.
    pub summary: String,
    /// Event timestamp supplied by caller or external input.
    pub occurred_at_ms: u64,
}

impl RunPlanRecordedJournalRecord {
    /// Creates a plan-recorded journal record request.
    #[must_use]
    pub fn new(run_id: RunId, summary: impl Into<String>, occurred_at_ms: u64) -> Self {
        Self {
            run_id,
            summary: summary.into(),
            occurred_at_ms,
        }
    }

    /// Converts this request into the corresponding control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        ControlEvent::run(
            self.run_id,
            self.occurred_at_ms,
            ControlEventKind::PlanRecorded {
                summary: self.summary,
            },
        )
    }
}

/// Records one plan summary fact as an append-only control event.
///
/// # Errors
///
/// Returns a control error when the ledger append fails.
pub fn record_run_plan_recorded<L>(
    ledger: &L,
    request: RunPlanRecordedJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    ledger.append_event(request.into_event())
}

/// Terminal run status to record.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RunTerminalJournalStatus {
    /// The run completed.
    Completed,
    /// The run failed.
    Failed {
        /// Failure message.
        message: String,
    },
    /// The run was blocked.
    Blocked {
        /// Block reason.
        reason: String,
    },
    /// The run was aborted.
    Aborted {
        /// Abort reason.
        reason: String,
    },
}

/// Named request for recording one terminal run fact.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RunTerminalJournalRecord {
    /// Owning run id.
    pub run_id: RunId,
    /// Terminal status to record.
    pub status: RunTerminalJournalStatus,
    /// Event timestamp supplied by caller or external input.
    pub occurred_at_ms: u64,
}

impl RunTerminalJournalRecord {
    /// Creates a completed terminal run journal record request.
    #[must_use]
    pub const fn completed(run_id: RunId, occurred_at_ms: u64) -> Self {
        Self {
            run_id,
            status: RunTerminalJournalStatus::Completed,
            occurred_at_ms,
        }
    }

    /// Creates a failed terminal run journal record request.
    #[must_use]
    pub fn failed(run_id: RunId, message: impl Into<String>, occurred_at_ms: u64) -> Self {
        Self {
            run_id,
            status: RunTerminalJournalStatus::Failed {
                message: message.into(),
            },
            occurred_at_ms,
        }
    }

    /// Creates a blocked terminal run journal record request.
    #[must_use]
    pub fn blocked(run_id: RunId, reason: impl Into<String>, occurred_at_ms: u64) -> Self {
        Self {
            run_id,
            status: RunTerminalJournalStatus::Blocked {
                reason: reason.into(),
            },
            occurred_at_ms,
        }
    }

    /// Creates an aborted terminal run journal record request.
    #[must_use]
    pub fn aborted(run_id: RunId, reason: impl Into<String>, occurred_at_ms: u64) -> Self {
        Self {
            run_id,
            status: RunTerminalJournalStatus::Aborted {
                reason: reason.into(),
            },
            occurred_at_ms,
        }
    }

    /// Converts this request into the corresponding control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        let kind = match self.status {
            RunTerminalJournalStatus::Completed => ControlEventKind::RunCompleted,
            RunTerminalJournalStatus::Failed { message } => ControlEventKind::RunFailed { message },
            RunTerminalJournalStatus::Blocked { reason } => ControlEventKind::RunBlocked { reason },
            RunTerminalJournalStatus::Aborted { reason } => ControlEventKind::RunAborted { reason },
        };
        ControlEvent::run(self.run_id, self.occurred_at_ms, kind)
    }
}

/// Records one terminal run fact as an append-only control event.
///
/// # Errors
///
/// Returns a control error when the ledger append fails.
pub fn record_run_terminal<L>(
    ledger: &L,
    request: RunTerminalJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    ledger.append_event(request.into_event())
}
