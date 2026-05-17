//! Deterministic event replay into run and step views.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ArtifactRef, Budget, ControlError, ControlEventKind, ControlEventRecord, ControlResult,
    CostObservation, EvidenceRef, GateResult, RecoveryAttempt, RunId, RunStatus, StepId, StepLease,
    StepStatus, WaitReason,
};

/// Current replayed view of one run.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RunView {
    /// Run id.
    pub run_id: RunId,
    /// Run lifecycle status.
    pub status: RunStatus,
    /// Original intent when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    /// Optional run budget.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget: Option<Budget>,
    /// Replayed steps by step id.
    #[serde(default)]
    pub steps: BTreeMap<StepId, StepView>,
    /// Run-level artifacts.
    #[serde(default)]
    pub artifacts: Vec<ArtifactRef>,
    /// Run-level cost observations.
    #[serde(default)]
    pub cost_observations: Vec<CostObservation>,
    /// Last update timestamp.
    #[serde(default)]
    pub updated_at_ms: u64,
}

/// Current replayed view of one step.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StepView {
    /// Step id.
    pub step_id: StepId,
    /// Step lifecycle status.
    pub status: StepStatus,
    /// Human-readable title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Required evidence keys.
    #[serde(default)]
    pub required_evidence: Vec<String>,
    /// Optional step budget.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget: Option<Budget>,
    /// Active lease when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_lease: Option<StepLease>,
    /// Current wait reason.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wait_reason: Option<WaitReason>,
    /// Attached evidence.
    #[serde(default)]
    pub evidence: Vec<EvidenceRef>,
    /// Attached artifacts.
    #[serde(default)]
    pub artifacts: Vec<ArtifactRef>,
    /// Cost observations.
    #[serde(default)]
    pub cost_observations: Vec<CostObservation>,
    /// Gate results.
    #[serde(default)]
    pub gate_results: Vec<GateResult>,
    /// Recovery attempts.
    #[serde(default)]
    pub recovery_attempts: Vec<RecoveryAttempt>,
    /// Last error message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    /// Last update timestamp.
    #[serde(default)]
    pub updated_at_ms: u64,
}

impl StepView {
    fn new(step_id: StepId) -> Self {
        Self {
            step_id,
            status: StepStatus::Pending,
            title: None,
            required_evidence: Vec::new(),
            budget: None,
            active_lease: None,
            wait_reason: None,
            evidence: Vec::new(),
            artifacts: Vec::new(),
            cost_observations: Vec::new(),
            gate_results: Vec::new(),
            recovery_attempts: Vec::new(),
            last_error: None,
            updated_at_ms: 0,
        }
    }

    /// Returns total observed cost in USD micros.
    #[must_use]
    pub fn total_cost_usd_micros(&self) -> u64 {
        self.cost_observations
            .iter()
            .map(|observation| observation.cost_usd_micros)
            .sum()
    }

    /// Returns covered required evidence keys in stable order.
    #[must_use]
    pub fn covered_required_evidence(&self) -> Vec<String> {
        self.evidence
            .iter()
            .filter_map(|evidence| evidence.requirement_key.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}

impl RunView {
    fn new(run_id: RunId) -> Self {
        Self {
            run_id,
            status: RunStatus::Draft,
            intent: None,
            budget: None,
            steps: BTreeMap::new(),
            artifacts: Vec::new(),
            cost_observations: Vec::new(),
            updated_at_ms: 0,
        }
    }

    /// Returns total observed cost in USD micros across run and step rows.
    #[must_use]
    pub fn total_cost_usd_micros(&self) -> u64 {
        self.cost_observations
            .iter()
            .map(|observation| observation.cost_usd_micros)
            .sum::<u64>()
            + self
                .steps
                .values()
                .map(StepView::total_cost_usd_micros)
                .sum::<u64>()
    }
}

/// Replays stored events into one run view.
///
/// # Errors
///
/// Returns a control error when no events are supplied or events from multiple
/// runs are mixed.
pub fn replay_run_view(records: Vec<ControlEventRecord>) -> ControlResult<RunView> {
    let Some(first) = records.first() else {
        return Err(ControlError::InvalidEventSequence {
            message: "cannot replay run view from empty event list".to_string(),
        });
    };
    let run_id = first.event.run_id.clone();
    let mut view = RunView::new(run_id.clone());
    let mut records = records;
    records.sort_by_key(|record| record.sequence);
    for record in records {
        if record.event.run_id != run_id {
            return Err(ControlError::InvalidEventSequence {
                message: "cannot replay events from multiple run ids".to_string(),
            });
        }
        apply_event(&mut view, record)?;
    }
    Ok(view)
}

fn apply_event(view: &mut RunView, record: ControlEventRecord) -> ControlResult<()> {
    let occurred_at_ms = record.event.occurred_at_ms;
    view.updated_at_ms = view.updated_at_ms.max(occurred_at_ms);
    match (record.event.step_id, record.event.kind) {
        (None, kind) => apply_run_event(view, kind),
        (Some(step_id), kind) => {
            let step = view
                .steps
                .entry(step_id.clone())
                .or_insert_with(|| StepView::new(step_id));
            step.updated_at_ms = step.updated_at_ms.max(occurred_at_ms);
            apply_step_event(step, kind);
            Ok(())
        }
    }
}

fn apply_run_event(view: &mut RunView, kind: ControlEventKind) -> ControlResult<()> {
    match kind {
        ControlEventKind::RunCreated { intent, budget, .. } => {
            view.intent = Some(intent);
            view.budget = budget;
            view.status = RunStatus::Draft;
        }
        ControlEventKind::RunAdmitted => view.status = RunStatus::Admitted,
        ControlEventKind::PlanRecorded { .. } => view.status = RunStatus::Planned,
        ControlEventKind::ArtifactAttached { artifact } => view.artifacts.push(artifact),
        ControlEventKind::CostObserved { observation } => view.cost_observations.push(observation),
        ControlEventKind::RecoveryStarted { .. } => view.status = RunStatus::Recovering,
        ControlEventKind::RunCompleted => view.status = RunStatus::Completed,
        ControlEventKind::RunFailed { .. } => view.status = RunStatus::Failed,
        ControlEventKind::RunBlocked { .. } => view.status = RunStatus::Blocked,
        ControlEventKind::RunAborted { .. } => view.status = RunStatus::Aborted,
        other => {
            return Err(ControlError::InvalidEventSequence {
                message: format!("step-scoped event `{other:?}` cannot be replayed at run scope"),
            });
        }
    }
    Ok(())
}

fn apply_step_event(step: &mut StepView, kind: ControlEventKind) {
    match kind {
        ControlEventKind::StepCreated {
            title,
            required_evidence,
            budget,
        } => {
            step.title = Some(title);
            step.required_evidence = required_evidence;
            step.budget = budget;
            step.status = StepStatus::Pending;
        }
        ControlEventKind::StepQueued => step.status = StepStatus::Queued,
        ControlEventKind::StepLeaseAcquired { lease }
        | ControlEventKind::StepLeaseRenewed { lease } => {
            step.active_lease = Some(lease);
            step.status = StepStatus::Leased;
        }
        ControlEventKind::StepLeaseReleased { .. } => {
            step.active_lease = None;
            step.status = StepStatus::Queued;
        }
        ControlEventKind::StepStarted => step.status = StepStatus::Running,
        ControlEventKind::StepWaiting { reason } => {
            step.wait_reason = Some(reason);
            step.status = StepStatus::Waiting;
        }
        ControlEventKind::ArtifactAttached { artifact } => step.artifacts.push(artifact),
        ControlEventKind::EvidenceAttached { evidence } => step.evidence.push(evidence),
        ControlEventKind::CostObserved { observation } => step.cost_observations.push(observation),
        ControlEventKind::GateEvaluated { result } => step.gate_results.push(result),
        ControlEventKind::RecoveryStarted { attempt } => {
            step.recovery_attempts.push(attempt);
            step.status = StepStatus::Recovering;
        }
        ControlEventKind::StepSucceeded => step.status = StepStatus::Succeeded,
        ControlEventKind::StepFailed { message, .. } => {
            step.last_error = Some(message);
            step.status = StepStatus::Failed;
        }
        ControlEventKind::StepBlocked { reason } => {
            step.last_error = Some(reason);
            step.status = StepStatus::Blocked;
        }
        ControlEventKind::StepCancelled { reason } => {
            step.last_error = Some(reason);
            step.status = StepStatus::Cancelled;
        }
        ControlEventKind::ToolCallRecorded { .. }
        | ControlEventKind::WorkerHeartbeatObserved { .. }
        | ControlEventKind::RunCreated { .. }
        | ControlEventKind::RunAdmitted
        | ControlEventKind::PlanRecorded { .. }
        | ControlEventKind::RunCompleted
        | ControlEventKind::RunFailed { .. }
        | ControlEventKind::RunBlocked { .. }
        | ControlEventKind::RunAborted { .. } => {}
    }
}
