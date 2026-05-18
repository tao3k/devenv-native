use std::collections::BTreeMap;

use serde_json::json;
use xiuxian_qianji_control::{
    ControlError, ControlEvent, ControlEventKind, ControlEventRecord, ControlLedger, ControlResult,
    RunId, RunStatus, RunView, StepId,
};

use crate::workflow_kernel::{WorkflowStageStatus, WorkflowStageTrace, WorkflowTrace};

const WORKFLOW_KERNEL_SOURCE: &str = "xiuxian_qianji.workflow_kernel";
const WORKFLOW_STAGE_TOOL_NAME: &str = "workflow_kernel_stage";

/// Duplicate-run policy for workflow control recording.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum WorkflowControlRecordingPolicy {
    /// Reject recording when the target ledger already contains events for
    /// the workflow run id.
    #[default]
    RejectExistingRun,
    /// Append projected events even when the ledger already contains events
    /// for the workflow run id.
    AppendOnly,
    /// Reuse the existing replayed run when the ledger already contains
    /// events for the workflow run id.
    ReuseExistingRun,
}

/// Summary returned after recording a workflow trace into a control ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowControlRecordingOutcome {
    /// Control-plane run id derived from the workflow id.
    pub run_id: RunId,
    /// Terminal run status derived from the trace.
    pub terminal_status: RunStatus,
    /// Number of events appended by this recording call.
    pub appended_event_count: usize,
    /// Ledger records returned by append operations.
    pub records: Vec<ControlEventRecord>,
    /// Ledger-replayed view after recording completed.
    pub run_view: RunView,
}

/// Workflow-side required-evidence declarations for control projection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkflowControlEvidenceRequirements {
    required_by_stage: BTreeMap<StepId, Vec<String>>,
}

impl WorkflowControlEvidenceRequirements {
    /// Creates an empty requirements set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            required_by_stage: BTreeMap::new(),
        }
    }

    /// Returns true when no stage requirements are declared.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.required_by_stage.is_empty()
    }

    /// Adds required-evidence keys for one workflow stage.
    ///
    /// # Errors
    ///
    /// Returns a control error when the stage id or any required-evidence key
    /// is blank.
    pub fn require_stage_evidence<I, S>(
        mut self,
        stage_id: impl Into<String>,
        required_evidence: I,
    ) -> ControlResult<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.insert_stage_evidence(stage_id, required_evidence)?;
        Ok(self)
    }

    /// Inserts or replaces required-evidence keys for one workflow stage.
    ///
    /// # Errors
    ///
    /// Returns a control error when the stage id or any required-evidence key
    /// is blank.
    pub fn insert_stage_evidence<I, S>(
        &mut self,
        stage_id: impl Into<String>,
        required_evidence: I,
    ) -> ControlResult<()>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let stage_id = StepId::new(stage_id)?;
        let required_evidence = normalize_required_evidence(required_evidence)?;
        self.required_by_stage.insert(stage_id, required_evidence);
        Ok(())
    }

    fn required_evidence_for(&self, step_id: &StepId) -> Vec<String> {
        self.required_by_stage
            .get(step_id)
            .cloned()
            .unwrap_or_default()
    }
}

/// Managed recorder for workflow trace control projections.
#[derive(Clone, Copy)]
pub struct WorkflowControlRecorder<'ledger> {
    ledger: &'ledger dyn ControlLedger,
    policy: WorkflowControlRecordingPolicy,
    required_evidence: Option<&'ledger WorkflowControlEvidenceRequirements>,
}

impl std::fmt::Debug for WorkflowControlRecorder<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorkflowControlRecorder")
            .field("policy", &self.policy)
            .finish_non_exhaustive()
    }
}

impl<'ledger> WorkflowControlRecorder<'ledger> {
    /// Creates a recorder with the default duplicate-run policy.
    #[must_use]
    pub const fn new(ledger: &'ledger dyn ControlLedger) -> Self {
        Self {
            ledger,
            policy: WorkflowControlRecordingPolicy::RejectExistingRun,
            required_evidence: None,
        }
    }

    /// Returns a recorder using the supplied duplicate-run policy.
    #[must_use]
    pub const fn with_policy(mut self, policy: WorkflowControlRecordingPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Returns a recorder that projects workflow stage required evidence.
    #[must_use]
    pub const fn with_required_evidence(
        mut self,
        required_evidence: &'ledger WorkflowControlEvidenceRequirements,
    ) -> Self {
        self.required_evidence = Some(required_evidence);
        self
    }

    /// Projects and appends a workflow trace into the configured ledger.
    ///
    /// # Errors
    ///
    /// Returns a control error when projection fails, when the default
    /// duplicate-run policy rejects an existing run, or when the ledger
    /// rejects an append operation.
    pub fn record_trace(
        self,
        trace: &WorkflowTrace,
    ) -> ControlResult<WorkflowControlRecordingOutcome> {
        let run_id = RunId::new(trace.workflow_id.clone())?;
        let existing_records = self.ledger.load_events(&run_id)?;
        if !existing_records.is_empty() {
            match self.policy {
                WorkflowControlRecordingPolicy::RejectExistingRun => {
                    return Err(ControlError::InvalidEventSequence {
                        message: format!(
                            "workflow control recording rejected existing run `{}`",
                            run_id.as_str()
                        ),
                    });
                }
                WorkflowControlRecordingPolicy::AppendOnly => {}
                WorkflowControlRecordingPolicy::ReuseExistingRun => {
                    let run_view = self.ledger.load_run_view(&run_id)?;
                    return Ok(WorkflowControlRecordingOutcome {
                        run_id,
                        terminal_status: workflow_trace_terminal_status(trace),
                        appended_event_count: 0,
                        records: Vec::new(),
                        run_view,
                    });
                }
            }
        }
        let terminal_status = workflow_trace_terminal_status(trace);
        let records = workflow_trace_to_control_events_with_optional_required_evidence(
            trace,
            self.required_evidence,
        )?
        .into_iter()
        .map(|event| self.ledger.append_event(event))
        .collect::<ControlResult<Vec<_>>>()?;
        let run_view = self.ledger.load_run_view(&run_id)?;
        Ok(WorkflowControlRecordingOutcome {
            run_id,
            terminal_status,
            appended_event_count: records.len(),
            records,
            run_view,
        })
    }
}

/// Maps a workflow trace into generic Qianji control-plane events.
///
/// # Errors
///
/// Returns a control error when the workflow id or any stage id is blank.
pub fn workflow_trace_to_control_events(trace: &WorkflowTrace) -> ControlResult<Vec<ControlEvent>> {
    workflow_trace_to_control_events_with_optional_required_evidence(trace, None)
}

/// Maps a workflow trace into control-plane events with stage evidence requirements.
///
/// # Errors
///
/// Returns a control error when the workflow id or any stage id is blank, or
/// when requirements reference a stage that is not present in the trace.
pub fn workflow_trace_to_control_events_with_required_evidence(
    trace: &WorkflowTrace,
    required_evidence: &WorkflowControlEvidenceRequirements,
) -> ControlResult<Vec<ControlEvent>> {
    workflow_trace_to_control_events_with_optional_required_evidence(trace, Some(required_evidence))
}

fn workflow_trace_to_control_events_with_optional_required_evidence(
    trace: &WorkflowTrace,
    required_evidence: Option<&WorkflowControlEvidenceRequirements>,
) -> ControlResult<Vec<ControlEvent>> {
    let run_id = RunId::new(trace.workflow_id.clone())?;
    validate_required_evidence_trace_coverage(trace, required_evidence)?;
    let started_at_ms = trace
        .stages
        .first()
        .map_or(0, |stage| stage.started_unix_ms);
    let mut events = vec![
        ControlEvent::run(
            run_id.clone(),
            started_at_ms,
            ControlEventKind::RunCreated {
                intent: format!("workflow:{}", trace.workflow_id),
                budget: None,
                metadata: json!({
                    "source": WORKFLOW_KERNEL_SOURCE,
                    "stageCount": trace.stages.len(),
                }),
            },
        ),
        ControlEvent::run(run_id.clone(), started_at_ms, ControlEventKind::RunAdmitted),
        ControlEvent::run(
            run_id.clone(),
            started_at_ms,
            ControlEventKind::PlanRecorded {
                summary: format!("Workflow trace with {} stage(s)", trace.stages.len()),
            },
        ),
    ];

    for stage in &trace.stages {
        append_stage_events(&mut events, &run_id, stage, required_evidence)?;
    }

    let terminal_at_ms = trace
        .stages
        .last()
        .map_or(started_at_ms, stage_terminal_at_ms);
    if let Some(failed_stage) = trace
        .stages
        .iter()
        .find(|stage| stage.status == WorkflowStageStatus::Failed)
    {
        events.push(ControlEvent::run(
            run_id,
            terminal_at_ms,
            ControlEventKind::RunFailed {
                message: failed_stage
                    .error
                    .clone()
                    .unwrap_or_else(|| "workflow stage failed".to_owned()),
            },
        ));
    } else {
        events.push(ControlEvent::run(
            run_id,
            terminal_at_ms,
            ControlEventKind::RunCompleted,
        ));
    }

    Ok(events)
}

/// Maps a workflow trace into sequence-numbered control records for immediate replay.
///
/// # Errors
///
/// Returns a control error when event mapping fails.
pub fn workflow_trace_to_control_event_records(
    trace: &WorkflowTrace,
) -> ControlResult<Vec<ControlEventRecord>> {
    workflow_trace_to_control_events(trace).map(|events| {
        events
            .into_iter()
            .enumerate()
            .map(|(index, event)| ControlEventRecord {
                sequence: u64::try_from(index).unwrap_or(u64::MAX).saturating_add(1),
                event,
            })
            .collect()
    })
}

/// Maps a workflow trace and stage evidence requirements into sequence-numbered records.
///
/// # Errors
///
/// Returns a control error when event mapping fails.
pub fn workflow_trace_to_control_event_records_with_required_evidence(
    trace: &WorkflowTrace,
    required_evidence: &WorkflowControlEvidenceRequirements,
) -> ControlResult<Vec<ControlEventRecord>> {
    workflow_trace_to_control_events_with_required_evidence(trace, required_evidence).map(
        |events| {
            events
                .into_iter()
                .enumerate()
                .map(|(index, event)| ControlEventRecord {
                    sequence: u64::try_from(index).unwrap_or(u64::MAX).saturating_add(1),
                    event,
                })
                .collect()
        },
    )
}

/// Projects and appends a workflow trace into an injected control ledger.
///
/// # Errors
///
/// Returns a control error when projection fails or when the ledger rejects an
/// append operation.
pub fn record_workflow_trace_to_control_ledger(
    ledger: &dyn ControlLedger,
    trace: &WorkflowTrace,
) -> ControlResult<Vec<ControlEventRecord>> {
    WorkflowControlRecorder::new(ledger)
        .with_policy(WorkflowControlRecordingPolicy::AppendOnly)
        .record_trace(trace)
        .map(|outcome| outcome.records)
}

/// Projects and appends a workflow trace with required evidence into a ledger.
///
/// # Errors
///
/// Returns a control error when projection fails or when the ledger rejects an
/// append operation.
pub fn record_workflow_trace_to_control_ledger_with_required_evidence(
    ledger: &dyn ControlLedger,
    trace: &WorkflowTrace,
    required_evidence: &WorkflowControlEvidenceRequirements,
) -> ControlResult<Vec<ControlEventRecord>> {
    WorkflowControlRecorder::new(ledger)
        .with_policy(WorkflowControlRecordingPolicy::AppendOnly)
        .with_required_evidence(required_evidence)
        .record_trace(trace)
        .map(|outcome| outcome.records)
}

fn append_stage_events(
    events: &mut Vec<ControlEvent>,
    run_id: &RunId,
    stage: &WorkflowStageTrace,
    required_evidence: Option<&WorkflowControlEvidenceRequirements>,
) -> ControlResult<()> {
    let step_id = StepId::new(stage.stage_id.clone())?;
    let terminal_at_ms = stage_terminal_at_ms(stage);
    let required_evidence = required_evidence
        .map(|requirements| requirements.required_evidence_for(&step_id))
        .unwrap_or_default();
    events.push(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        stage.started_unix_ms,
        ControlEventKind::StepCreated {
            title: stage.stage_id.clone(),
            required_evidence,
            budget: None,
        },
    ));
    events.push(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        stage.started_unix_ms,
        ControlEventKind::StepStarted,
    ));
    events.push(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        terminal_at_ms,
        ControlEventKind::ToolCallRecorded {
            tool_name: WORKFLOW_STAGE_TOOL_NAME.to_owned(),
            metadata: stage_metadata(stage),
        },
    ));
    match stage.status {
        WorkflowStageStatus::Succeeded => events.push(ControlEvent::step(
            run_id.clone(),
            step_id,
            terminal_at_ms,
            ControlEventKind::StepSucceeded,
        )),
        WorkflowStageStatus::Failed => events.push(ControlEvent::step(
            run_id.clone(),
            step_id,
            terminal_at_ms,
            ControlEventKind::StepFailed {
                error_code: "workflow_stage_failed".to_owned(),
                message: stage
                    .error
                    .clone()
                    .unwrap_or_else(|| "workflow stage failed".to_owned()),
                retryable: false,
            },
        )),
    }
    Ok(())
}

fn workflow_trace_terminal_status(trace: &WorkflowTrace) -> RunStatus {
    if trace
        .stages
        .iter()
        .any(|stage| stage.status == WorkflowStageStatus::Failed)
    {
        RunStatus::Failed
    } else {
        RunStatus::Completed
    }
}

fn stage_metadata(stage: &WorkflowStageTrace) -> serde_json::Value {
    json!({
        "source": WORKFLOW_KERNEL_SOURCE,
        "stageId": stage.stage_id,
        "status": stage.status,
        "durationNanos": stage.duration_nanos,
        "input": stage.input,
        "output": stage.output,
        "checkpoints": stage.checkpoints,
    })
}

fn normalize_required_evidence<I, S>(required_evidence: I) -> ControlResult<Vec<String>>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut normalized = Vec::new();
    for key in required_evidence {
        let key = key.into().trim().to_owned();
        if key.is_empty() {
            return Err(ControlError::BlankId {
                field: "required_evidence",
            });
        }
        if !normalized.iter().any(|existing| existing == &key) {
            normalized.push(key);
        }
    }
    Ok(normalized)
}

fn validate_required_evidence_trace_coverage(
    trace: &WorkflowTrace,
    required_evidence: Option<&WorkflowControlEvidenceRequirements>,
) -> ControlResult<()> {
    let Some(required_evidence) = required_evidence else {
        return Ok(());
    };
    for stage_id in required_evidence.required_by_stage.keys() {
        if !trace
            .stages
            .iter()
            .any(|stage| stage.stage_id == stage_id.as_str())
        {
            return Err(ControlError::InvalidEventSequence {
                message: format!(
                    "required evidence declared for unknown workflow stage `{}`",
                    stage_id.as_str()
                ),
            });
        }
    }
    Ok(())
}

fn stage_terminal_at_ms(stage: &WorkflowStageTrace) -> u64 {
    stage
        .started_unix_ms
        .saturating_add(stage.duration_nanos / 1_000_000)
}
