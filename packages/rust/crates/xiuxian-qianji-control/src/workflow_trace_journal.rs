//! Workflow-neutral projection journal for complete workflow traces.

use crate::{
    ControlError, ControlEvent, ControlEventRecord, ControlLedger, ControlResult,
    RunAdmittedJournalRecord, RunCreatedJournalRecord, RunId, RunPlanRecordedJournalRecord,
    RunTerminalJournalRecord, StepCreatedJournalRecord, StepFailureJournalInput, StepId,
    StepStartedJournalRecord, StepTerminalJournalRecord, StepToolCallJournalRecord,
    record_control_event_batch,
};

/// Terminal status for one projected workflow stage.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WorkflowTraceProjectionStageStatus {
    /// The workflow stage succeeded.
    Succeeded,
    /// The workflow stage failed.
    Failed {
        /// Stable error code.
        error_code: String,
        /// Human-readable failure message.
        message: String,
        /// Whether the failure is retryable.
        retryable: bool,
    },
}

/// Control-plane projection for one workflow stage.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkflowTraceProjectionStage {
    /// Control-plane step id for the workflow stage.
    pub step_id: StepId,
    /// Human-readable step title.
    pub title: String,
    /// Stage start timestamp in Unix milliseconds.
    pub started_at_ms: u64,
    /// Stage terminal timestamp in Unix milliseconds.
    pub terminal_at_ms: u64,
    /// Required evidence keys for the stage.
    #[serde(default)]
    pub required_evidence: Vec<String>,
    /// Tool name used to preserve the stage execution fact.
    pub tool_name: String,
    /// Extension metadata for the tool-call record.
    #[serde(default)]
    pub metadata: serde_json::Value,
    /// Stage terminal status.
    pub status: WorkflowTraceProjectionStageStatus,
}

/// Input for creating one projected workflow stage.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkflowTraceProjectionStageInput {
    /// Control-plane step id for the workflow stage.
    pub step_id: StepId,
    /// Human-readable step title.
    pub title: String,
    /// Stage start timestamp in Unix milliseconds.
    pub started_at_ms: u64,
    /// Stage terminal timestamp in Unix milliseconds.
    pub terminal_at_ms: u64,
    /// Tool name used to preserve the stage execution fact.
    pub tool_name: String,
    /// Extension metadata for the tool-call record.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl WorkflowTraceProjectionStageInput {
    /// Creates stage projection input with null metadata and zero timestamps.
    #[must_use]
    pub fn new(step_id: StepId, title: impl Into<String>, tool_name: impl Into<String>) -> Self {
        Self {
            step_id,
            title: title.into(),
            started_at_ms: 0,
            terminal_at_ms: 0,
            tool_name: tool_name.into(),
            metadata: serde_json::Value::Null,
        }
    }

    /// Sets stage timestamps.
    #[must_use]
    pub const fn with_timestamps(mut self, started_at_ms: u64, terminal_at_ms: u64) -> Self {
        self.started_at_ms = started_at_ms;
        self.terminal_at_ms = terminal_at_ms;
        self
    }

    /// Sets extension metadata for the tool-call record.
    #[must_use]
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

impl WorkflowTraceProjectionStage {
    /// Creates a successful stage projection.
    #[must_use]
    pub fn succeeded(input: WorkflowTraceProjectionStageInput) -> Self {
        let WorkflowTraceProjectionStageInput {
            step_id,
            title,
            started_at_ms,
            terminal_at_ms,
            tool_name,
            metadata,
        } = input;
        Self {
            step_id,
            title,
            started_at_ms,
            terminal_at_ms,
            required_evidence: Vec::new(),
            tool_name,
            metadata,
            status: WorkflowTraceProjectionStageStatus::Succeeded,
        }
    }

    /// Creates a failed stage projection.
    #[must_use]
    pub fn failed(input: WorkflowTraceProjectionStageInput, message: impl Into<String>) -> Self {
        let WorkflowTraceProjectionStageInput {
            step_id,
            title,
            started_at_ms,
            terminal_at_ms,
            tool_name,
            metadata,
        } = input;
        Self {
            step_id,
            title,
            started_at_ms,
            terminal_at_ms,
            required_evidence: Vec::new(),
            tool_name,
            metadata,
            status: WorkflowTraceProjectionStageStatus::Failed {
                error_code: "workflow_stage_failed".to_owned(),
                message: message.into(),
                retryable: false,
            },
        }
    }

    /// Sets required evidence keys for this stage.
    #[must_use]
    pub fn with_required_evidence(mut self, required_evidence: Vec<String>) -> Self {
        self.required_evidence = required_evidence;
        self
    }
}

/// Control-plane projection for a complete workflow trace.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkflowTraceProjectionRecord {
    /// Control-plane run id.
    pub run_id: RunId,
    /// Human-readable run intent.
    pub intent: String,
    /// Run start timestamp in Unix milliseconds.
    pub started_at_ms: u64,
    /// Run-created extension metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
    /// Human-readable plan summary.
    pub plan_summary: String,
    /// Ordered stage projections.
    #[serde(default)]
    pub stages: Vec<WorkflowTraceProjectionStage>,
}

impl WorkflowTraceProjectionRecord {
    /// Creates a workflow trace projection.
    #[must_use]
    pub fn new(run_id: RunId, intent: impl Into<String>, started_at_ms: u64) -> Self {
        Self {
            run_id,
            intent: intent.into(),
            started_at_ms,
            metadata: serde_json::Value::Null,
            plan_summary: String::new(),
            stages: Vec::new(),
        }
    }

    /// Sets run-created metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Sets the plan summary.
    #[must_use]
    pub fn with_plan_summary(mut self, summary: impl Into<String>) -> Self {
        self.plan_summary = summary.into();
        self
    }

    /// Sets ordered stage projections.
    #[must_use]
    pub fn with_stages(mut self, stages: Vec<WorkflowTraceProjectionStage>) -> Self {
        self.stages = stages;
        self
    }

    /// Converts this projection into replayable control events.
    ///
    /// # Errors
    ///
    /// Returns a control error when a required tool name, stage failure
    /// message, or required evidence key is blank.
    pub fn into_events(self) -> ControlResult<Vec<ControlEvent>> {
        validate_projection(&self)?;

        let Self {
            run_id,
            intent,
            started_at_ms,
            metadata,
            plan_summary,
            stages,
        } = self;
        let terminal_at_ms = stages
            .last()
            .map_or(started_at_ms, |stage| stage.terminal_at_ms);
        let failed_message = stages.iter().find_map(|stage| {
            if let WorkflowTraceProjectionStageStatus::Failed { message, .. } = &stage.status {
                Some(message.clone())
            } else {
                None
            }
        });

        let mut events = vec![
            RunCreatedJournalRecord::new(run_id.clone(), intent, started_at_ms)
                .with_metadata(metadata)
                .into_event(),
            RunAdmittedJournalRecord::new(run_id.clone(), started_at_ms).into_event(),
            RunPlanRecordedJournalRecord::new(run_id.clone(), plan_summary, started_at_ms)
                .into_event(),
        ];

        for stage in stages {
            append_stage_events(&mut events, &run_id, stage);
        }

        if let Some(message) = failed_message {
            events.push(
                RunTerminalJournalRecord::failed(run_id, message, terminal_at_ms).into_event(),
            );
        } else {
            events.push(RunTerminalJournalRecord::completed(run_id, terminal_at_ms).into_event());
        }

        Ok(events)
    }
}

/// Records a workflow trace projection as an append-only event batch.
///
/// # Errors
///
/// Returns a control error when projection validation fails or when the ledger
/// rejects an append operation.
pub fn record_workflow_trace_projection<L>(
    ledger: &L,
    projection: WorkflowTraceProjectionRecord,
) -> ControlResult<Vec<ControlEventRecord>>
where
    L: ControlLedger + ?Sized,
{
    record_control_event_batch(ledger, projection.into_events()?).map(|outcome| outcome.records)
}

fn validate_projection(projection: &WorkflowTraceProjectionRecord) -> ControlResult<()> {
    if projection.intent.trim().is_empty() {
        return Err(ControlError::InvalidEventSequence {
            message: "workflow trace projection intent cannot be blank".to_owned(),
        });
    }
    if projection.plan_summary.trim().is_empty() {
        return Err(ControlError::InvalidEventSequence {
            message: "workflow trace projection plan summary cannot be blank".to_owned(),
        });
    }
    for stage in &projection.stages {
        if stage.tool_name.trim().is_empty() {
            return Err(ControlError::InvalidEventSequence {
                message: format!(
                    "workflow trace projection stage `{}` has a blank tool name",
                    stage.step_id.as_str()
                ),
            });
        }
        if stage
            .required_evidence
            .iter()
            .any(|evidence| evidence.trim().is_empty())
        {
            return Err(ControlError::InvalidEventSequence {
                message: format!(
                    "workflow trace projection stage `{}` has a blank required evidence key",
                    stage.step_id.as_str()
                ),
            });
        }
        if let WorkflowTraceProjectionStageStatus::Failed { message, .. } = &stage.status
            && message.trim().is_empty()
        {
            return Err(ControlError::InvalidEventSequence {
                message: format!(
                    "workflow trace projection stage `{}` has a blank failure message",
                    stage.step_id.as_str()
                ),
            });
        }
    }
    Ok(())
}

fn append_stage_events(
    events: &mut Vec<ControlEvent>,
    run_id: &RunId,
    stage: WorkflowTraceProjectionStage,
) {
    let WorkflowTraceProjectionStage {
        step_id,
        title,
        started_at_ms,
        terminal_at_ms,
        required_evidence,
        tool_name,
        metadata,
        status,
    } = stage;

    events.push(
        StepCreatedJournalRecord::new(run_id.clone(), step_id.clone(), title, started_at_ms)
            .with_required_evidence(required_evidence)
            .into_event(),
    );
    events.push(
        StepStartedJournalRecord::new(run_id.clone(), step_id.clone(), started_at_ms).into_event(),
    );
    events.push(
        StepToolCallJournalRecord::new(run_id.clone(), step_id.clone(), tool_name, terminal_at_ms)
            .with_metadata(metadata)
            .into_event(),
    );
    match status {
        WorkflowTraceProjectionStageStatus::Succeeded => events.push(
            StepTerminalJournalRecord::succeeded(run_id.clone(), step_id, terminal_at_ms)
                .into_event(),
        ),
        WorkflowTraceProjectionStageStatus::Failed {
            error_code,
            message,
            retryable,
        } => events.push(
            StepTerminalJournalRecord::failed(
                run_id.clone(),
                step_id,
                StepFailureJournalInput::new(error_code, message, retryable),
                terminal_at_ms,
            )
            .into_event(),
        ),
    }
}
