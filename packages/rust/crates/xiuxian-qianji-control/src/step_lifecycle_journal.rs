//! Step lifecycle journal recording helpers.

use crate::{
    Budget, ControlEvent, ControlEventKind, ControlEventRecord, ControlLedger, ControlResult,
    RunId, StepId,
};

/// Named request for recording one step-created fact.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StepCreatedJournalRecord {
    /// Owning run id.
    pub run_id: RunId,
    /// Owning step id.
    pub step_id: StepId,
    /// Human-readable step title.
    pub title: String,
    /// Required evidence keys for this step.
    #[serde(default)]
    pub required_evidence: Vec<String>,
    /// Optional step budget.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget: Option<Budget>,
    /// Event timestamp supplied by caller or external input.
    pub occurred_at_ms: u64,
}

impl StepCreatedJournalRecord {
    /// Creates a step-created journal record request.
    #[must_use]
    pub fn new(
        run_id: RunId,
        step_id: StepId,
        title: impl Into<String>,
        occurred_at_ms: u64,
    ) -> Self {
        Self {
            run_id,
            step_id,
            title: title.into(),
            required_evidence: Vec::new(),
            budget: None,
            occurred_at_ms,
        }
    }

    /// Sets required evidence keys for this step.
    #[must_use]
    pub fn with_required_evidence(mut self, required_evidence: Vec<String>) -> Self {
        self.required_evidence = required_evidence;
        self
    }

    /// Sets the optional step budget.
    #[must_use]
    pub fn with_budget(mut self, budget: Budget) -> Self {
        self.budget = Some(budget);
        self
    }

    /// Converts this request into the corresponding control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        ControlEvent::step(
            self.run_id,
            self.step_id,
            self.occurred_at_ms,
            ControlEventKind::StepCreated {
                title: self.title,
                required_evidence: self.required_evidence,
                budget: self.budget,
            },
        )
    }
}

/// Records one step-created fact as an append-only control event.
///
/// # Errors
///
/// Returns a control error when the ledger append fails.
pub fn record_step_created<L>(
    ledger: &L,
    request: StepCreatedJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    ledger.append_event(request.into_event())
}

/// Named request for recording one step-started fact.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StepStartedJournalRecord {
    /// Owning run id.
    pub run_id: RunId,
    /// Owning step id.
    pub step_id: StepId,
    /// Event timestamp supplied by caller or external input.
    pub occurred_at_ms: u64,
}

impl StepStartedJournalRecord {
    /// Creates a step-started journal record request.
    #[must_use]
    pub const fn new(run_id: RunId, step_id: StepId, occurred_at_ms: u64) -> Self {
        Self {
            run_id,
            step_id,
            occurred_at_ms,
        }
    }

    /// Converts this request into the corresponding control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        ControlEvent::step(
            self.run_id,
            self.step_id,
            self.occurred_at_ms,
            ControlEventKind::StepStarted,
        )
    }
}

/// Records one step-started fact as an append-only control event.
///
/// # Errors
///
/// Returns a control error when the ledger append fails.
pub fn record_step_started<L>(
    ledger: &L,
    request: StepStartedJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    ledger.append_event(request.into_event())
}

/// Named request for recording one step tool-call fact.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StepToolCallJournalRecord {
    /// Owning run id.
    pub run_id: RunId,
    /// Owning step id.
    pub step_id: StepId,
    /// Tool name.
    pub tool_name: String,
    /// Extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
    /// Event timestamp supplied by caller or external input.
    pub occurred_at_ms: u64,
}

impl StepToolCallJournalRecord {
    /// Creates a step tool-call journal record request.
    #[must_use]
    pub fn new(
        run_id: RunId,
        step_id: StepId,
        tool_name: impl Into<String>,
        occurred_at_ms: u64,
    ) -> Self {
        Self {
            run_id,
            step_id,
            tool_name: tool_name.into(),
            metadata: serde_json::Value::Null,
            occurred_at_ms,
        }
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
        ControlEvent::step(
            self.run_id,
            self.step_id,
            self.occurred_at_ms,
            ControlEventKind::ToolCallRecorded {
                tool_name: self.tool_name,
                metadata: self.metadata,
            },
        )
    }
}

/// Records one step tool-call fact as an append-only control event.
///
/// # Errors
///
/// Returns a control error when the ledger append fails.
pub fn record_step_tool_call<L>(
    ledger: &L,
    request: StepToolCallJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    ledger.append_event(request.into_event())
}

/// Terminal step status to record.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StepTerminalJournalStatus {
    /// The step succeeded.
    Succeeded,
    /// The step failed.
    Failed {
        /// Stable error code.
        error_code: String,
        /// Human-readable failure message.
        message: String,
        /// Whether the failure is retryable.
        retryable: bool,
    },
    /// The step was blocked.
    Blocked {
        /// Block reason.
        reason: String,
    },
    /// The step was cancelled.
    Cancelled {
        /// Cancel reason.
        reason: String,
    },
}

/// Failure payload for a terminal step journal record.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StepFailureJournalInput {
    /// Stable error code.
    pub error_code: String,
    /// Human-readable failure message.
    pub message: String,
    /// Whether the failure is retryable.
    pub retryable: bool,
}

impl StepFailureJournalInput {
    /// Creates a step failure payload.
    #[must_use]
    pub fn new(error_code: impl Into<String>, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            error_code: error_code.into(),
            message: message.into(),
            retryable,
        }
    }
}

/// Named request for recording one terminal step fact.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StepTerminalJournalRecord {
    /// Owning run id.
    pub run_id: RunId,
    /// Owning step id.
    pub step_id: StepId,
    /// Terminal status to record.
    pub status: StepTerminalJournalStatus,
    /// Event timestamp supplied by caller or external input.
    pub occurred_at_ms: u64,
}

impl StepTerminalJournalRecord {
    /// Creates a successful terminal step journal record request.
    #[must_use]
    pub const fn succeeded(run_id: RunId, step_id: StepId, occurred_at_ms: u64) -> Self {
        Self {
            run_id,
            step_id,
            status: StepTerminalJournalStatus::Succeeded,
            occurred_at_ms,
        }
    }

    /// Creates a failed terminal step journal record request.
    #[must_use]
    pub fn failed(
        run_id: RunId,
        step_id: StepId,
        failure: StepFailureJournalInput,
        occurred_at_ms: u64,
    ) -> Self {
        Self {
            run_id,
            step_id,
            status: StepTerminalJournalStatus::Failed {
                error_code: failure.error_code,
                message: failure.message,
                retryable: failure.retryable,
            },
            occurred_at_ms,
        }
    }

    /// Creates a blocked terminal step journal record request.
    #[must_use]
    pub fn blocked(
        run_id: RunId,
        step_id: StepId,
        reason: impl Into<String>,
        occurred_at_ms: u64,
    ) -> Self {
        Self {
            run_id,
            step_id,
            status: StepTerminalJournalStatus::Blocked {
                reason: reason.into(),
            },
            occurred_at_ms,
        }
    }

    /// Creates a cancelled terminal step journal record request.
    #[must_use]
    pub fn cancelled(
        run_id: RunId,
        step_id: StepId,
        reason: impl Into<String>,
        occurred_at_ms: u64,
    ) -> Self {
        Self {
            run_id,
            step_id,
            status: StepTerminalJournalStatus::Cancelled {
                reason: reason.into(),
            },
            occurred_at_ms,
        }
    }

    /// Converts this request into the corresponding control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        let kind = match self.status {
            StepTerminalJournalStatus::Succeeded => ControlEventKind::StepSucceeded,
            StepTerminalJournalStatus::Failed {
                error_code,
                message,
                retryable,
            } => ControlEventKind::StepFailed {
                error_code,
                message,
                retryable,
            },
            StepTerminalJournalStatus::Blocked { reason } => {
                ControlEventKind::StepBlocked { reason }
            }
            StepTerminalJournalStatus::Cancelled { reason } => {
                ControlEventKind::StepCancelled { reason }
            }
        };
        ControlEvent::step(self.run_id, self.step_id, self.occurred_at_ms, kind)
    }
}

/// Records one terminal step fact as an append-only control event.
///
/// # Errors
///
/// Returns a control error when the ledger append fails.
pub fn record_step_terminal<L>(
    ledger: &L,
    request: StepTerminalJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    ledger.append_event(request.into_event())
}
