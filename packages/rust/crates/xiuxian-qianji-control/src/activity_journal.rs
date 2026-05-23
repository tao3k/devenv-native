//! Activity schedule journal recording helpers.

use crate::{
    ActivityFailure, ActivityId, ActivityResult, ActivityStatus, ActivityTask, ActivityView,
    ControlError, ControlEvent, ControlEventKind, ControlEventRecord, ControlLedger, ControlResult,
    LlmActivityAdmission, RunId, RunView, StepId, ToolActivityAdmission, WorkerId, replay_run_view,
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
    fn appended(record: ControlEventRecord) -> Self {
        Self {
            status: ActivityJournalWriteStatus::Appended,
            record,
        }
    }

    fn already_recorded(record: ControlEventRecord) -> Self {
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

    fn run_id(&self) -> &RunId {
        match self {
            Self::Run { run_id } | Self::Step { run_id, .. } => run_id,
        }
    }

    fn step_id(&self) -> Option<&StepId> {
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
}

/// Records an already admitted tool activity as an `ActivityScheduled` event.
///
/// # Errors
///
/// Returns a control error when the admission payload is invalid or the ledger
/// append fails.
pub fn record_admitted_activity_schedule<L>(
    ledger: &L,
    request: AdmittedActivityScheduleRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    request.admission.validate()?;
    let AdmittedActivityScheduleRecord {
        run_id,
        step_id,
        occurred_at_ms,
        admission,
    } = request;
    let event_kind = ControlEventKind::ActivityScheduled {
        task: admission.task,
    };
    let event = match step_id {
        Some(step_id) => ControlEvent::step(run_id, step_id, occurred_at_ms, event_kind),
        None => ControlEvent::run(run_id, occurred_at_ms, event_kind),
    };
    ledger.append_event(event)
}

/// Records an admitted activity schedule with duplicate and transition guards.
///
/// # Errors
///
/// Returns a control error when the admission payload is invalid, the activity
/// was already scheduled with different details, replay fails, or the ledger
/// append fails.
pub fn record_admitted_activity_schedule_idempotent<L>(
    ledger: &L,
    request: AdmittedActivityScheduleRecord,
) -> ControlResult<ActivityJournalWriteOutcome>
where
    L: ControlLedger + ?Sized,
{
    request.admission.validate()?;
    let run_id = request.run_id.clone();
    let step_id = request.step_id.clone();
    let task = request.admission.task.clone();
    let kind = ControlEventKind::ActivityScheduled { task: task.clone() };
    let records = ledger.load_events(&run_id)?;
    if let Some(record) = find_existing_activity_event(&records, &run_id, step_id.as_ref(), &kind) {
        return Ok(ActivityJournalWriteOutcome::already_recorded(record));
    }
    let view = replay_run_view(records)?;
    validate_schedule_transition(&view, step_id.as_ref(), &task)?;
    record_admitted_activity_schedule(ledger, request).map(ActivityJournalWriteOutcome::appended)
}

/// Records an already admitted workflow-neutral activity task as an
/// `ActivityScheduled` event.
///
/// # Errors
///
/// Returns a control error when the task payload is invalid or the ledger
/// append fails.
pub fn record_admitted_activity_task_schedule<L>(
    ledger: &L,
    request: AdmittedActivityTaskScheduleRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    request.task.validate()?;
    let AdmittedActivityTaskScheduleRecord {
        run_id,
        step_id,
        occurred_at_ms,
        task,
    } = request;
    let event_kind = ControlEventKind::ActivityScheduled { task };
    let event = match step_id {
        Some(step_id) => ControlEvent::step(run_id, step_id, occurred_at_ms, event_kind),
        None => ControlEvent::run(run_id, occurred_at_ms, event_kind),
    };
    ledger.append_event(event)
}

/// Records an admitted workflow-neutral activity task schedule with duplicate
/// and transition guards.
///
/// # Errors
///
/// Returns a control error when the task payload is invalid, the activity was
/// already scheduled with different details, replay fails, or the ledger append
/// fails.
pub fn record_admitted_activity_task_schedule_idempotent<L>(
    ledger: &L,
    request: AdmittedActivityTaskScheduleRecord,
) -> ControlResult<ActivityJournalWriteOutcome>
where
    L: ControlLedger + ?Sized,
{
    request.task.validate()?;
    let run_id = request.run_id.clone();
    let step_id = request.step_id.clone();
    let task = request.task.clone();
    let kind = ControlEventKind::ActivityScheduled { task: task.clone() };
    let records = ledger.load_events(&run_id)?;
    if let Some(record) = find_existing_activity_event(&records, &run_id, step_id.as_ref(), &kind) {
        return Ok(ActivityJournalWriteOutcome::already_recorded(record));
    }
    let view = replay_run_view(records)?;
    validate_schedule_transition(&view, step_id.as_ref(), &task)?;
    record_admitted_activity_task_schedule(ledger, request)
        .map(ActivityJournalWriteOutcome::appended)
}

/// Records an already admitted LLM activity as an `ActivityScheduled` event.
///
/// # Errors
///
/// Returns a control error when the admission payload is invalid or the ledger
/// append fails.
pub fn record_admitted_llm_activity_schedule<L>(
    ledger: &L,
    request: AdmittedLlmActivityScheduleRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    request.admission.validate()?;
    let AdmittedLlmActivityScheduleRecord {
        run_id,
        step_id,
        occurred_at_ms,
        admission,
    } = request;
    let event_kind = ControlEventKind::ActivityScheduled {
        task: llm_activity_schedule_task(&admission),
    };
    let event = match step_id {
        Some(step_id) => ControlEvent::step(run_id, step_id, occurred_at_ms, event_kind),
        None => ControlEvent::run(run_id, occurred_at_ms, event_kind),
    };
    ledger.append_event(event)
}

/// Records an admitted LLM activity schedule with duplicate and transition guards.
///
/// # Errors
///
/// Returns a control error when the admission payload is invalid, the activity
/// was already scheduled with different details, replay fails, or the ledger
/// append fails.
pub fn record_admitted_llm_activity_schedule_idempotent<L>(
    ledger: &L,
    request: AdmittedLlmActivityScheduleRecord,
) -> ControlResult<ActivityJournalWriteOutcome>
where
    L: ControlLedger + ?Sized,
{
    request.admission.validate()?;
    let run_id = request.run_id.clone();
    let step_id = request.step_id.clone();
    let task = llm_activity_schedule_task(&request.admission);
    let kind = ControlEventKind::ActivityScheduled { task: task.clone() };
    let records = ledger.load_events(&run_id)?;
    if let Some(record) = find_existing_activity_event(&records, &run_id, step_id.as_ref(), &kind) {
        return Ok(ActivityJournalWriteOutcome::already_recorded(record));
    }
    let view = replay_run_view(records)?;
    validate_schedule_transition(&view, step_id.as_ref(), &task)?;
    record_admitted_llm_activity_schedule(ledger, request)
        .map(ActivityJournalWriteOutcome::appended)
}

/// Records an activity attempt start as an `ActivityStarted` event.
///
/// # Errors
///
/// Returns a control error when the attempt is zero or the ledger append fails.
pub fn record_activity_started<L>(
    ledger: &L,
    request: ActivityStartedJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    validate_started_record(&request)?;
    let ActivityStartedJournalRecord {
        scope,
        occurred_at_ms,
        activity_id,
        worker_id,
        attempt,
    } = request;
    ledger.append_event(scope.into_event(
        occurred_at_ms,
        ControlEventKind::ActivityStarted {
            activity_id,
            worker_id,
            attempt,
        },
    ))
}

/// Records an activity attempt start with duplicate and transition guards.
///
/// # Errors
///
/// Returns a control error when the start record is invalid, the activity was
/// not scheduled, the transition is invalid, replay fails, or the ledger append
/// fails.
pub fn record_activity_started_idempotent<L>(
    ledger: &L,
    request: ActivityStartedJournalRecord,
) -> ControlResult<ActivityJournalWriteOutcome>
where
    L: ControlLedger + ?Sized,
{
    validate_started_record(&request)?;
    let scope = request.scope.clone();
    let activity_id = request.activity_id.clone();
    let worker_id = request.worker_id.clone();
    let attempt = request.attempt;
    let kind = ControlEventKind::ActivityStarted {
        activity_id: activity_id.clone(),
        worker_id,
        attempt,
    };
    let records = ledger.load_events(scope.run_id())?;
    if let Some(record) =
        find_existing_activity_event(&records, scope.run_id(), scope.step_id(), &kind)
    {
        return Ok(ActivityJournalWriteOutcome::already_recorded(record));
    }
    let view = replay_run_view(records)?;
    validate_start_transition(&view, scope.step_id(), &activity_id, attempt)?;
    record_activity_started(ledger, request).map(ActivityJournalWriteOutcome::appended)
}

/// Records an activity completion as an `ActivityCompleted` event.
///
/// # Errors
///
/// Returns a control error when the result payload is invalid or the ledger
/// append fails.
pub fn record_activity_completed<L>(
    ledger: &L,
    request: ActivityCompletedJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    validate_result(&request.result)?;
    let ActivityCompletedJournalRecord {
        scope,
        occurred_at_ms,
        activity_id,
        result,
    } = request;
    ledger.append_event(scope.into_event(
        occurred_at_ms,
        ControlEventKind::ActivityCompleted {
            activity_id,
            result,
        },
    ))
}

/// Records an activity completion with duplicate and transition guards.
///
/// # Errors
///
/// Returns a control error when the result payload is invalid, the activity is
/// not in a started state, replay fails, or the ledger append fails.
pub fn record_activity_completed_idempotent<L>(
    ledger: &L,
    request: ActivityCompletedJournalRecord,
) -> ControlResult<ActivityJournalWriteOutcome>
where
    L: ControlLedger + ?Sized,
{
    validate_result(&request.result)?;
    let scope = request.scope.clone();
    let activity_id = request.activity_id.clone();
    let kind = ControlEventKind::ActivityCompleted {
        activity_id: activity_id.clone(),
        result: request.result.clone(),
    };
    let records = ledger.load_events(scope.run_id())?;
    if let Some(record) =
        find_existing_activity_event(&records, scope.run_id(), scope.step_id(), &kind)
    {
        return Ok(ActivityJournalWriteOutcome::already_recorded(record));
    }
    let view = replay_run_view(records)?;
    validate_completion_transition(&view, scope.step_id(), &activity_id)?;
    record_activity_completed(ledger, request).map(ActivityJournalWriteOutcome::appended)
}

/// Records an activity failure as an `ActivityFailed` event.
///
/// # Errors
///
/// Returns a control error when the failure payload is invalid or the ledger
/// append fails.
pub fn record_activity_failed<L>(
    ledger: &L,
    request: ActivityFailedJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    validate_failure(&request.failure)?;
    let ActivityFailedJournalRecord {
        scope,
        occurred_at_ms,
        activity_id,
        failure,
    } = request;
    ledger.append_event(scope.into_event(
        occurred_at_ms,
        ControlEventKind::ActivityFailed {
            activity_id,
            failure,
        },
    ))
}

/// Records an activity failure with duplicate and transition guards.
///
/// # Errors
///
/// Returns a control error when the failure payload is invalid, the activity
/// state cannot accept the failure, replay fails, or the ledger append fails.
pub fn record_activity_failed_idempotent<L>(
    ledger: &L,
    request: ActivityFailedJournalRecord,
) -> ControlResult<ActivityJournalWriteOutcome>
where
    L: ControlLedger + ?Sized,
{
    validate_failure(&request.failure)?;
    let scope = request.scope.clone();
    let activity_id = request.activity_id.clone();
    let failure = request.failure.clone();
    let kind = ControlEventKind::ActivityFailed {
        activity_id: activity_id.clone(),
        failure: failure.clone(),
    };
    let records = ledger.load_events(scope.run_id())?;
    if let Some(record) =
        find_existing_activity_event(&records, scope.run_id(), scope.step_id(), &kind)
    {
        return Ok(ActivityJournalWriteOutcome::already_recorded(record));
    }
    let view = replay_run_view(records)?;
    validate_failure_transition(&view, scope.step_id(), &activity_id, &failure)?;
    record_activity_failed(ledger, request).map(ActivityJournalWriteOutcome::appended)
}

const LLM_ACTIVITY_REQUEST_AUDIT_METADATA_KEY: &str = "qianji_llm_activity_request";
const ORIGINAL_ACTIVITY_METADATA_KEY: &str = "qianji_original_activity_metadata";

fn llm_activity_schedule_task(admission: &LlmActivityAdmission) -> ActivityTask {
    let mut task = admission.activity_task().clone();
    task.metadata =
        with_llm_request_audit_metadata(task.metadata, llm_request_audit_metadata(admission));
    task
}

fn with_llm_request_audit_metadata(
    existing_metadata: serde_json::Value,
    audit_metadata: serde_json::Value,
) -> serde_json::Value {
    match existing_metadata {
        serde_json::Value::Object(mut metadata) => {
            metadata.insert(
                LLM_ACTIVITY_REQUEST_AUDIT_METADATA_KEY.to_owned(),
                audit_metadata,
            );
            serde_json::Value::Object(metadata)
        }
        serde_json::Value::Null => serde_json::json!({
            LLM_ACTIVITY_REQUEST_AUDIT_METADATA_KEY: audit_metadata,
        }),
        metadata => serde_json::json!({
            LLM_ACTIVITY_REQUEST_AUDIT_METADATA_KEY: audit_metadata,
            ORIGINAL_ACTIVITY_METADATA_KEY: metadata,
        }),
    }
}

fn llm_request_audit_metadata(admission: &LlmActivityAdmission) -> serde_json::Value {
    let request = &admission.activity.request;
    serde_json::json!({
        "schema": "qianji.llm_activity_request_audit.v1",
        "model": request.model.as_str(),
        "prompt_ref": &request.prompt_ref,
        "context_ref": &request.context_ref,
        "tool_schema_hash": &request.tool_schema_hash,
        "temperature_millis": request.temperature_millis,
        "max_tokens": request.max_tokens,
        "response_schema_ref": &request.response_schema_ref,
        "budget": &request.budget,
        "request_metadata": &request.metadata,
        "admission_metadata": &admission.metadata,
    })
}

fn find_existing_activity_event(
    records: &[ControlEventRecord],
    run_id: &RunId,
    step_id: Option<&StepId>,
    kind: &ControlEventKind,
) -> Option<ControlEventRecord> {
    records
        .iter()
        .find(|record| {
            &record.event.run_id == run_id
                && record.event.step_id.as_ref() == step_id
                && &record.event.kind == kind
        })
        .cloned()
}

fn validate_schedule_transition(
    view: &RunView,
    step_id: Option<&StepId>,
    task: &ActivityTask,
) -> ControlResult<()> {
    if activity_for_scope(view, step_id, &task.activity_id).is_some() {
        return Err(invalid_activity_journal(
            "activity schedule already exists for activity_id with different lifecycle history",
        ));
    }
    Ok(())
}

fn validate_start_transition(
    view: &RunView,
    step_id: Option<&StepId>,
    activity_id: &ActivityId,
    attempt: u32,
) -> ControlResult<()> {
    let activity = required_activity_for_scope(view, step_id, activity_id, "start")?;
    match activity.status {
        ActivityStatus::Scheduled => Ok(()),
        ActivityStatus::Failed if attempt > activity.attempt => Ok(()),
        ActivityStatus::Started => Err(invalid_activity_journal(
            "activity start is already in progress; duplicate starts must match an existing event",
        )),
        ActivityStatus::Completed => Err(invalid_activity_journal(
            "activity start cannot follow a completed activity",
        )),
        ActivityStatus::Failed => Err(invalid_activity_journal(
            "activity retry start attempt must be greater than the failed attempt",
        )),
        ActivityStatus::Pending => Err(invalid_activity_journal(
            "activity start requires a scheduled activity",
        )),
    }
}

fn validate_completion_transition(
    view: &RunView,
    step_id: Option<&StepId>,
    activity_id: &ActivityId,
) -> ControlResult<()> {
    let activity = required_activity_for_scope(view, step_id, activity_id, "completion")?;
    match activity.status {
        ActivityStatus::Started => Ok(()),
        ActivityStatus::Completed => Err(invalid_activity_journal(
            "activity completion is already recorded with different result",
        )),
        ActivityStatus::Failed => Err(invalid_activity_journal(
            "activity completion cannot follow a failed activity",
        )),
        ActivityStatus::Pending | ActivityStatus::Scheduled => Err(invalid_activity_journal(
            "activity completion requires a started activity",
        )),
    }
}

fn validate_failure_transition(
    view: &RunView,
    step_id: Option<&StepId>,
    activity_id: &ActivityId,
    failure: &ActivityFailure,
) -> ControlResult<()> {
    let activity = required_activity_for_scope(view, step_id, activity_id, "failure")?;
    match activity.status {
        ActivityStatus::Scheduled => Ok(()),
        ActivityStatus::Started if failure.attempt == activity.attempt => Ok(()),
        ActivityStatus::Started => Err(invalid_activity_journal(
            "activity failure attempt must match the started attempt",
        )),
        ActivityStatus::Failed => Err(invalid_activity_journal(
            "activity failure is already recorded with different payload",
        )),
        ActivityStatus::Completed => Err(invalid_activity_journal(
            "activity failure cannot follow a completed activity",
        )),
        ActivityStatus::Pending => Err(invalid_activity_journal(
            "activity failure requires a scheduled activity",
        )),
    }
}

fn required_activity_for_scope<'a>(
    view: &'a RunView,
    step_id: Option<&StepId>,
    activity_id: &ActivityId,
    transition: &str,
) -> ControlResult<&'a ActivityView> {
    activity_for_scope(view, step_id, activity_id).ok_or_else(|| {
        invalid_activity_journal(&format!(
            "activity {transition} requires a scheduled activity"
        ))
    })
}

fn activity_for_scope<'a>(
    view: &'a RunView,
    step_id: Option<&StepId>,
    activity_id: &ActivityId,
) -> Option<&'a ActivityView> {
    match step_id {
        Some(step_id) => view
            .steps
            .get(step_id)
            .and_then(|step| step.activities.get(activity_id)),
        None => view.activities.get(activity_id),
    }
}

fn validate_started_record(request: &ActivityStartedJournalRecord) -> ControlResult<()> {
    if request.attempt == 0 {
        return Err(invalid_activity_journal(
            "activity started attempt must be at least 1",
        ));
    }
    Ok(())
}

fn validate_result(result: &ActivityResult) -> ControlResult<()> {
    if result
        .output_hash
        .as_ref()
        .is_some_and(|hash| hash.trim().is_empty())
    {
        return Err(invalid_activity_journal(
            "activity result output_hash must not be blank when supplied",
        ));
    }
    Ok(())
}

fn validate_failure(failure: &ActivityFailure) -> ControlResult<()> {
    if failure.attempt == 0 {
        return Err(invalid_activity_journal(
            "activity failure attempt must be at least 1",
        ));
    }
    if failure.message.trim().is_empty() {
        return Err(invalid_activity_journal(
            "activity failure message must not be blank",
        ));
    }
    Ok(())
}

fn invalid_activity_journal(message: &str) -> ControlError {
    ControlError::InvalidEventSequence {
        message: message.to_owned(),
    }
}
