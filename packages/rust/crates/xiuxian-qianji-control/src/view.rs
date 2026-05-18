//! Deterministic event replay into run and step views.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ActivityFailure, ActivityId, ActivityResult, ActivityTask, AgentDecision, AgentDecisionId,
    AgentProposal, AgentProposalId, ArtifactRef, Budget, ControlError, ControlEventKind,
    ControlEventRecord, ControlResult, CostObservation, EvidenceRef, GateResult, RecoveryAttempt,
    RunId, RunStatus, SignalRecord, StepId, StepLease, StepStatus, TimerId, TimerRecord,
    VersionKey, VersionPin, WaitReason, WorkerId,
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
    /// Run-scoped activities by activity id.
    #[serde(default)]
    pub activities: BTreeMap<ActivityId, ActivityView>,
    /// Run-scoped Agent proposals by proposal id.
    #[serde(default)]
    pub agent_proposals: BTreeMap<AgentProposalId, AgentProposal>,
    /// Run-scoped Agent decisions by decision id.
    #[serde(default)]
    pub agent_decisions: BTreeMap<AgentDecisionId, AgentDecision>,
    /// Run-scoped received signals.
    #[serde(default)]
    pub signals: Vec<SignalRecord>,
    /// Run-scoped timers by timer id.
    #[serde(default)]
    pub timers: BTreeMap<TimerId, TimerView>,
    /// Run-scoped deterministic version pins.
    #[serde(default)]
    pub version_pins: BTreeMap<VersionKey, VersionPin>,
    /// Last update timestamp.
    #[serde(default)]
    pub updated_at_ms: u64,
}

/// Activity lifecycle status reconstructed from journal events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ActivityStatus {
    /// Activity has not been scheduled.
    #[default]
    Pending,
    /// Activity task was scheduled.
    Scheduled,
    /// Activity worker started an attempt.
    Started,
    /// Activity completed successfully.
    Completed,
    /// Activity failed.
    Failed,
}

/// Replayed view of one activity.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivityView {
    /// Activity id.
    pub activity_id: ActivityId,
    /// Activity lifecycle status.
    pub status: ActivityStatus,
    /// Scheduled task details, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<ActivityTask>,
    /// Worker that started the latest attempt, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worker_id: Option<WorkerId>,
    /// Latest observed attempt number.
    #[serde(default)]
    pub attempt: u32,
    /// Successful result, when completed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<ActivityResult>,
    /// Failure payload, when failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<ActivityFailure>,
    /// Last update timestamp.
    #[serde(default)]
    pub updated_at_ms: u64,
}

impl ActivityView {
    fn new(activity_id: ActivityId) -> Self {
        Self {
            activity_id,
            status: ActivityStatus::Pending,
            task: None,
            worker_id: None,
            attempt: 0,
            result: None,
            failure: None,
            updated_at_ms: 0,
        }
    }
}

/// Timer lifecycle status reconstructed from journal events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TimerStatus {
    /// Timer has not been scheduled.
    #[default]
    Pending,
    /// Timer is waiting to fire.
    Scheduled,
    /// Timer fired.
    Fired,
}

/// Replayed view of one durable timer.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TimerView {
    /// Timer id.
    pub timer_id: TimerId,
    /// Timer lifecycle status.
    pub status: TimerStatus,
    /// Scheduled timer details, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timer: Option<TimerRecord>,
    /// Fire timestamp, when fired.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fired_at_ms: Option<u64>,
    /// Last update timestamp.
    #[serde(default)]
    pub updated_at_ms: u64,
}

impl TimerView {
    fn new(timer_id: TimerId) -> Self {
        Self {
            timer_id,
            status: TimerStatus::Pending,
            timer: None,
            fired_at_ms: None,
            updated_at_ms: 0,
        }
    }
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
    /// Step-scoped activities by activity id.
    #[serde(default)]
    pub activities: BTreeMap<ActivityId, ActivityView>,
    /// Step-scoped Agent proposals by proposal id.
    #[serde(default)]
    pub agent_proposals: BTreeMap<AgentProposalId, AgentProposal>,
    /// Step-scoped Agent decisions by decision id.
    #[serde(default)]
    pub agent_decisions: BTreeMap<AgentDecisionId, AgentDecision>,
    /// Step-scoped received signals.
    #[serde(default)]
    pub signals: Vec<SignalRecord>,
    /// Step-scoped timers by timer id.
    #[serde(default)]
    pub timers: BTreeMap<TimerId, TimerView>,
    /// Step-scoped deterministic version pins.
    #[serde(default)]
    pub version_pins: BTreeMap<VersionKey, VersionPin>,
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
            activities: BTreeMap::new(),
            agent_proposals: BTreeMap::new(),
            agent_decisions: BTreeMap::new(),
            signals: Vec::new(),
            timers: BTreeMap::new(),
            version_pins: BTreeMap::new(),
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
            activities: BTreeMap::new(),
            agent_proposals: BTreeMap::new(),
            agent_decisions: BTreeMap::new(),
            signals: Vec::new(),
            timers: BTreeMap::new(),
            version_pins: BTreeMap::new(),
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
        ControlEventKind::ActivityScheduled { task } => {
            apply_activity_scheduled(&mut view.activities, task, view.updated_at_ms);
        }
        ControlEventKind::AgentProposalRecorded { proposal } => {
            view.agent_proposals
                .insert(proposal.proposal_id.clone(), proposal);
        }
        ControlEventKind::AgentDecisionRecorded { decision } => {
            view.agent_decisions
                .insert(decision.decision_id.clone(), decision);
        }
        ControlEventKind::ActivityStarted {
            activity_id,
            worker_id,
            attempt,
        } => apply_activity_started(
            &mut view.activities,
            activity_id,
            worker_id,
            attempt,
            view.updated_at_ms,
        ),
        ControlEventKind::ActivityCompleted {
            activity_id,
            result,
        } => apply_activity_completed(
            &mut view.activities,
            activity_id,
            result,
            view.updated_at_ms,
        ),
        ControlEventKind::ActivityFailed {
            activity_id,
            failure,
        } => apply_activity_failed(
            &mut view.activities,
            activity_id,
            failure,
            view.updated_at_ms,
        ),
        ControlEventKind::SignalReceived { signal } => view.signals.push(signal),
        ControlEventKind::TimerScheduled { timer } => {
            apply_timer_scheduled(&mut view.timers, timer, view.updated_at_ms);
        }
        ControlEventKind::TimerFired { timer_id } => {
            apply_timer_fired(&mut view.timers, timer_id, view.updated_at_ms);
        }
        ControlEventKind::VersionPinned { pin } => {
            view.version_pins.insert(pin.version_key.clone(), pin);
        }
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
        ControlEventKind::StepWaiting { reason } => apply_step_waiting(step, reason),
        ControlEventKind::ArtifactAttached { artifact } => step.artifacts.push(artifact),
        ControlEventKind::EvidenceAttached { evidence } => step.evidence.push(evidence),
        ControlEventKind::CostObserved { observation } => step.cost_observations.push(observation),
        ControlEventKind::ActivityScheduled { task } => {
            apply_activity_scheduled(&mut step.activities, task, step.updated_at_ms);
        }
        ControlEventKind::AgentProposalRecorded { proposal } => {
            record_step_agent_proposal(step, proposal);
        }
        ControlEventKind::AgentDecisionRecorded { decision } => {
            record_step_agent_decision(step, decision);
        }
        ControlEventKind::ActivityStarted {
            activity_id,
            worker_id,
            attempt,
        } => apply_activity_started(
            &mut step.activities,
            activity_id,
            worker_id,
            attempt,
            step.updated_at_ms,
        ),
        ControlEventKind::ActivityCompleted {
            activity_id,
            result,
        } => apply_activity_completed(
            &mut step.activities,
            activity_id,
            result,
            step.updated_at_ms,
        ),
        ControlEventKind::ActivityFailed {
            activity_id,
            failure,
        } => apply_activity_failed(
            &mut step.activities,
            activity_id,
            failure,
            step.updated_at_ms,
        ),
        ControlEventKind::SignalReceived { signal } => step.signals.push(signal),
        ControlEventKind::TimerScheduled { timer } => {
            apply_timer_scheduled(&mut step.timers, timer, step.updated_at_ms);
        }
        ControlEventKind::TimerFired { timer_id } => {
            apply_timer_fired(&mut step.timers, timer_id, step.updated_at_ms);
        }
        ControlEventKind::VersionPinned { pin } => {
            step.version_pins.insert(pin.version_key.clone(), pin);
        }
        ControlEventKind::GateEvaluated { result } => step.gate_results.push(result),
        ControlEventKind::RecoveryStarted { attempt } => apply_step_recovery_started(step, attempt),
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

fn apply_step_waiting(step: &mut StepView, reason: WaitReason) {
    step.wait_reason = Some(reason);
    step.status = StepStatus::Waiting;
}

fn record_step_agent_proposal(step: &mut StepView, proposal: AgentProposal) {
    step.agent_proposals
        .insert(proposal.proposal_id.clone(), proposal);
}

fn record_step_agent_decision(step: &mut StepView, decision: AgentDecision) {
    step.agent_decisions
        .insert(decision.decision_id.clone(), decision);
}

fn apply_step_recovery_started(step: &mut StepView, attempt: RecoveryAttempt) {
    step.recovery_attempts.push(attempt);
    step.status = StepStatus::Recovering;
}

fn apply_activity_scheduled(
    activities: &mut BTreeMap<ActivityId, ActivityView>,
    task: ActivityTask,
    occurred_at_ms: u64,
) {
    let activity = activities
        .entry(task.activity_id.clone())
        .or_insert_with(|| ActivityView::new(task.activity_id.clone()));
    activity.task = Some(task);
    activity.status = ActivityStatus::Scheduled;
    activity.worker_id = None;
    activity.attempt = 0;
    activity.result = None;
    activity.failure = None;
    activity.updated_at_ms = activity.updated_at_ms.max(occurred_at_ms);
}

fn apply_activity_started(
    activities: &mut BTreeMap<ActivityId, ActivityView>,
    activity_id: ActivityId,
    worker_id: Option<WorkerId>,
    attempt: u32,
    occurred_at_ms: u64,
) {
    let activity = activities
        .entry(activity_id.clone())
        .or_insert_with(|| ActivityView::new(activity_id));
    activity.status = ActivityStatus::Started;
    activity.worker_id = worker_id;
    activity.attempt = attempt;
    activity.result = None;
    activity.failure = None;
    activity.updated_at_ms = activity.updated_at_ms.max(occurred_at_ms);
}

fn apply_activity_completed(
    activities: &mut BTreeMap<ActivityId, ActivityView>,
    activity_id: ActivityId,
    result: ActivityResult,
    occurred_at_ms: u64,
) {
    let activity = activities
        .entry(activity_id.clone())
        .or_insert_with(|| ActivityView::new(activity_id));
    activity.status = ActivityStatus::Completed;
    activity.result = Some(result);
    activity.failure = None;
    activity.updated_at_ms = activity.updated_at_ms.max(occurred_at_ms);
}

fn apply_activity_failed(
    activities: &mut BTreeMap<ActivityId, ActivityView>,
    activity_id: ActivityId,
    failure: ActivityFailure,
    occurred_at_ms: u64,
) {
    let activity = activities
        .entry(activity_id.clone())
        .or_insert_with(|| ActivityView::new(activity_id));
    activity.status = ActivityStatus::Failed;
    activity.attempt = failure.attempt;
    activity.result = None;
    activity.failure = Some(failure);
    activity.updated_at_ms = activity.updated_at_ms.max(occurred_at_ms);
}

fn apply_timer_scheduled(
    timers: &mut BTreeMap<TimerId, TimerView>,
    timer: TimerRecord,
    occurred_at_ms: u64,
) {
    let timer_view = timers
        .entry(timer.timer_id.clone())
        .or_insert_with(|| TimerView::new(timer.timer_id.clone()));
    timer_view.timer = Some(timer);
    timer_view.status = TimerStatus::Scheduled;
    timer_view.fired_at_ms = None;
    timer_view.updated_at_ms = timer_view.updated_at_ms.max(occurred_at_ms);
}

fn apply_timer_fired(
    timers: &mut BTreeMap<TimerId, TimerView>,
    timer_id: TimerId,
    occurred_at_ms: u64,
) {
    let timer_view = timers
        .entry(timer_id.clone())
        .or_insert_with(|| TimerView::new(timer_id));
    timer_view.status = TimerStatus::Fired;
    timer_view.fired_at_ms = Some(occurred_at_ms);
    timer_view.updated_at_ms = timer_view.updated_at_ms.max(occurred_at_ms);
}
