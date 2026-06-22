//! Activity schedule-admission plan contracts.

use std::collections::BTreeSet;

use crate::{
    ActivityId, ActivityJournalWriteStatus, ActivityTask, AdmittedActivityTaskScheduleRecord,
    ControlError, ControlLedger, ControlResult, RunId, StepId,
    record_admitted_activity_task_schedule_idempotent,
};

/// Supported Qianji schedule-admission plan contract.
pub const ACTIVITY_SCHEDULE_ADMISSION_PLAN_CONTRACT: &str =
    "xiuxian.qianji.control.activity_schedule_admission_plan.v1";
/// Supported schedule-admission row kind.
pub const ACTIVITY_SCHEDULE_ADMISSION_KIND: &str = "qianji_activity_schedule_admission_candidate";
/// Pending schedule-admission row status.
pub const ACTIVITY_SCHEDULE_ADMISSION_PENDING_STATUS: &str = "pending_qianji_admission";

/// Supported schedule-admission row kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ActivityScheduleAdmissionKind {
    /// Generic Qianji activity schedule-admission candidate.
    #[serde(rename = "qianji_activity_schedule_admission_candidate")]
    QianjiActivityScheduleAdmissionCandidate,
}

impl ActivityScheduleAdmissionKind {
    /// Returns the wire value for this admission kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::QianjiActivityScheduleAdmissionCandidate => ACTIVITY_SCHEDULE_ADMISSION_KIND,
        }
    }
}

/// Supported schedule-admission row status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ActivityScheduleAdmissionStatus {
    /// Pending Qianji admission.
    #[serde(rename = "pending_qianji_admission")]
    PendingQianjiAdmission,
}

impl ActivityScheduleAdmissionStatus {
    /// Returns the wire value for this admission status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PendingQianjiAdmission => ACTIVITY_SCHEDULE_ADMISSION_PENDING_STATUS,
        }
    }
}

/// One generic activity schedule-admission plan row.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityScheduleAdmissionPlanItem {
    /// Stable plan row id.
    pub schedule_item_id: String,
    /// Schedule-admission contract version.
    pub schedule_contract: String,
    /// Admission row kind.
    pub admission_kind: ActivityScheduleAdmissionKind,
    /// Qianji run id carried by the plan producer.
    pub qianji_run_id: String,
    /// Generic activity task to schedule durably.
    pub activity_task: ActivityTask,
    /// Execution safety flags.
    #[serde(flatten)]
    pub execution: ActivityScheduleAdmissionExecutionFlags,
    /// Non-promotion safety flags.
    #[serde(flatten)]
    pub safety: ActivityScheduleAdmissionSafetyFlags,
    /// Plan row status.
    pub status: ActivityScheduleAdmissionStatus,
}

/// Execution flags that must be false before admission.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityScheduleAdmissionExecutionFlags {
    /// Source/model execution flags.
    #[serde(flatten)]
    pub input: ActivityScheduleAdmissionInputExecutionFlags,
    /// Runtime execution flags.
    #[serde(flatten)]
    pub runtime: ActivityScheduleAdmissionRuntimeExecutionFlags,
}

/// Source/model execution flags that must be false before admission.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityScheduleAdmissionInputExecutionFlags {
    /// Whether source text was read by the plan producer.
    pub source_text_read: bool,
    /// Whether a live LLM was executed by the plan producer.
    pub llm_executed: bool,
}

/// Runtime execution flags that must be false before admission.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityScheduleAdmissionRuntimeExecutionFlags {
    /// Whether the workflow was executed by the plan producer.
    pub workflow_executed: bool,
    /// Whether Qianji ledger was mutated by the plan producer.
    pub qianji_ledger_mutated: bool,
    /// Whether hot-state work was enqueued by the plan producer.
    pub hot_state_enqueued: bool,
}

/// Safety flags that must be false before admission.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityScheduleAdmissionSafetyFlags {
    /// Whether the plan producer allowed source mutation.
    pub source_mutation_allowed: bool,
    /// Whether the plan producer allowed RDF mutation.
    pub rdf_mutation_allowed: bool,
    /// Whether the plan row claims ontology truth.
    pub ontology_truth: bool,
}

/// Request for admitting a generic activity schedule-admission plan.
#[derive(Debug, Clone, PartialEq)]
pub struct ActivitySchedulePlanAdmissionRequest {
    /// Owning run id.
    pub run_id: RunId,
    /// Optional owning step id.
    pub step_id: Option<StepId>,
    /// First event timestamp supplied by caller.
    pub occurred_at_ms: u64,
    /// Plan rows to admit.
    pub items: Vec<ActivityScheduleAdmissionPlanItem>,
}

impl ActivitySchedulePlanAdmissionRequest {
    /// Create a run-scoped schedule-plan admission request.
    #[must_use]
    pub fn run(
        run_id: RunId,
        occurred_at_ms: u64,
        items: Vec<ActivityScheduleAdmissionPlanItem>,
    ) -> Self {
        Self {
            run_id,
            step_id: None,
            occurred_at_ms,
            items,
        }
    }

    /// Create a step-scoped schedule-plan admission request.
    #[must_use]
    pub fn step(
        run_id: RunId,
        step_id: StepId,
        occurred_at_ms: u64,
        items: Vec<ActivityScheduleAdmissionPlanItem>,
    ) -> Self {
        Self {
            run_id,
            step_id: Some(step_id),
            occurred_at_ms,
            items,
        }
    }
}

/// Result of admitting one schedule-plan row.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivitySchedulePlanAdmissionItemOutcome {
    /// Stable plan row id.
    pub schedule_item_id: String,
    /// Activity id admitted for scheduling.
    pub activity_id: ActivityId,
    /// Idempotent write status.
    pub status: ActivityJournalWriteStatus,
    /// Durable ledger sequence for the stored event.
    pub sequence: u64,
}

/// Report returned after schedule-plan admission.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivitySchedulePlanAdmissionReport {
    /// Owning run id.
    pub run_id: RunId,
    /// Optional owning step id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<StepId>,
    /// Number of plan rows accepted for validation.
    pub plan_item_count: usize,
    /// Number of rows appended as new durable events.
    pub appended_count: usize,
    /// Number of rows already present as exact durable events.
    pub already_recorded_count: usize,
    /// Per-row admission outcomes.
    pub outcomes: Vec<ActivitySchedulePlanAdmissionItemOutcome>,
}

/// Parse a JSON schedule-admission plan.
///
/// # Errors
///
/// Returns a control codec error when JSON decoding fails.
pub fn parse_activity_schedule_plan_json(
    content: &str,
) -> ControlResult<Vec<ActivityScheduleAdmissionPlanItem>> {
    serde_json::from_str(content).map_err(|error| ControlError::Codec {
        operation: "parse_activity_schedule_plan_json",
        message: error.to_string(),
    })
}

/// Admit a generic activity schedule-admission plan into a durable ledger.
///
/// # Errors
///
/// Returns a control error when plan validation fails, idempotent schedule
/// recording fails, or timestamp arithmetic would overflow.
pub fn admit_activity_schedule_plan<L>(
    ledger: &L,
    request: ActivitySchedulePlanAdmissionRequest,
) -> ControlResult<ActivitySchedulePlanAdmissionReport>
where
    L: ControlLedger + ?Sized,
{
    validate_plan_request(&request)?;
    let ActivitySchedulePlanAdmissionRequest {
        run_id,
        step_id,
        occurred_at_ms,
        items,
    } = request;
    let outcomes = items
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            admit_activity_schedule_plan_item(
                ledger,
                &run_id,
                step_id.as_ref(),
                occurred_at_ms,
                index,
                item,
            )
        })
        .collect::<ControlResult<Vec<_>>>()?;
    let appended_count = outcomes
        .iter()
        .filter(|outcome| outcome.status == ActivityJournalWriteStatus::Appended)
        .count();
    let already_recorded_count = outcomes
        .iter()
        .filter(|outcome| outcome.status == ActivityJournalWriteStatus::AlreadyRecorded)
        .count();

    Ok(ActivitySchedulePlanAdmissionReport {
        run_id,
        step_id,
        plan_item_count: outcomes.len(),
        appended_count,
        already_recorded_count,
        outcomes,
    })
}

fn admit_activity_schedule_plan_item<L>(
    ledger: &L,
    run_id: &RunId,
    step_id: Option<&StepId>,
    occurred_at_ms: u64,
    index: usize,
    item: ActivityScheduleAdmissionPlanItem,
) -> ControlResult<ActivitySchedulePlanAdmissionItemOutcome>
where
    L: ControlLedger + ?Sized,
{
    let scheduled_at_ms = schedule_plan_item_timestamp(occurred_at_ms, index)?;
    let schedule_item_id = item.schedule_item_id;
    let activity_id = item.activity_task.activity_id.clone();
    let record = match step_id.cloned() {
        Some(step_id) => AdmittedActivityTaskScheduleRecord::step(
            run_id.clone(),
            step_id,
            scheduled_at_ms,
            item.activity_task,
        ),
        None => AdmittedActivityTaskScheduleRecord::run(
            run_id.clone(),
            scheduled_at_ms,
            item.activity_task,
        ),
    };
    let outcome = record_admitted_activity_task_schedule_idempotent(ledger, record)?;
    Ok(ActivitySchedulePlanAdmissionItemOutcome {
        schedule_item_id,
        activity_id,
        status: outcome.status,
        sequence: outcome.record.sequence,
    })
}

fn schedule_plan_item_timestamp(occurred_at_ms: u64, index: usize) -> ControlResult<u64> {
    let offset = u64::try_from(index).map_err(|_| ControlError::InvalidEventSequence {
        message: "schedule-plan admission timestamp index overflow".to_owned(),
    })?;
    occurred_at_ms
        .checked_add(offset)
        .ok_or_else(|| ControlError::InvalidEventSequence {
            message: "schedule-plan admission timestamp overflow".to_owned(),
        })
}

fn validate_plan_request(request: &ActivitySchedulePlanAdmissionRequest) -> ControlResult<()> {
    if request.items.is_empty() {
        return Err(invalid_schedule_plan(
            "activity schedule-admission plan has no rows",
        ));
    }
    let mut seen_schedule_item_ids = BTreeSet::new();
    let mut seen_activity_ids = BTreeSet::new();
    for item in &request.items {
        validate_plan_item(request.run_id.as_str(), item)?;
        if !seen_schedule_item_ids.insert(item.schedule_item_id.as_str()) {
            return Err(invalid_schedule_plan(format!(
                "duplicate schedule item id `{}`",
                item.schedule_item_id
            )));
        }
        if !seen_activity_ids.insert(item.activity_task.activity_id.as_str()) {
            return Err(invalid_schedule_plan(format!(
                "duplicate activity id `{}`",
                item.activity_task.activity_id.as_str()
            )));
        }
    }
    Ok(())
}

fn validate_plan_item(
    expected_run_id: &str,
    item: &ActivityScheduleAdmissionPlanItem,
) -> ControlResult<()> {
    if item.schedule_contract != ACTIVITY_SCHEDULE_ADMISSION_PLAN_CONTRACT {
        return Err(invalid_schedule_plan(format!(
            "schedule item `{}` has unsupported contract `{}`",
            item.schedule_item_id, item.schedule_contract
        )));
    }
    if item.admission_kind
        != ActivityScheduleAdmissionKind::QianjiActivityScheduleAdmissionCandidate
    {
        return Err(invalid_schedule_plan(format!(
            "schedule item `{}` has unsupported admission kind `{}`",
            item.schedule_item_id,
            item.admission_kind.as_str()
        )));
    }
    if item.qianji_run_id != expected_run_id {
        return Err(invalid_schedule_plan(format!(
            "schedule item `{}` run id `{}` does not match `{}`",
            item.schedule_item_id, item.qianji_run_id, expected_run_id
        )));
    }
    if item.status != ActivityScheduleAdmissionStatus::PendingQianjiAdmission {
        return Err(invalid_schedule_plan(format!(
            "schedule item `{}` has unsupported status `{}`",
            item.schedule_item_id,
            item.status.as_str()
        )));
    }
    validate_execution_flags(item)?;
    validate_safety_flags(item)?;
    item.activity_task.validate()?;
    if item.activity_task.input_ref.is_none() {
        return Err(invalid_schedule_plan(format!(
            "schedule item `{}` activity task requires input_ref",
            item.schedule_item_id
        )));
    }
    Ok(())
}

fn validate_execution_flags(item: &ActivityScheduleAdmissionPlanItem) -> ControlResult<()> {
    if item.execution.input.source_text_read {
        return Err(flag_error(item, "sourceTextRead"));
    }
    if item.execution.input.llm_executed {
        return Err(flag_error(item, "llmExecuted"));
    }
    if item.execution.runtime.workflow_executed {
        return Err(flag_error(item, "workflowExecuted"));
    }
    if item.execution.runtime.qianji_ledger_mutated {
        return Err(flag_error(item, "qianjiLedgerMutated"));
    }
    if item.execution.runtime.hot_state_enqueued {
        return Err(flag_error(item, "hotStateEnqueued"));
    }
    Ok(())
}

fn validate_safety_flags(item: &ActivityScheduleAdmissionPlanItem) -> ControlResult<()> {
    if item.safety.source_mutation_allowed {
        return Err(flag_error(item, "sourceMutationAllowed"));
    }
    if item.safety.rdf_mutation_allowed {
        return Err(flag_error(item, "rdfMutationAllowed"));
    }
    if item.safety.ontology_truth {
        return Err(flag_error(item, "ontologyTruth"));
    }
    Ok(())
}

fn flag_error(item: &ActivityScheduleAdmissionPlanItem, flag: &str) -> ControlError {
    invalid_schedule_plan(format!(
        "schedule item `{}` must keep `{flag}=false` before Qianji admission",
        item.schedule_item_id
    ))
}

fn invalid_schedule_plan(message: impl Into<String>) -> ControlError {
    ControlError::InvalidEventSequence {
        message: message.into(),
    }
}
