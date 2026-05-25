//! Flowhub BPMN `serviceTask` scheduling adapter for Qianji runtime execution.
//!
//! The adapter validates BPMN host-work identity and converts one pending
//! Flowhub service boundary into a workflow-neutral `ActivityTask` schedule
//! record for `xiuxian-qianji-control`.

use std::path::Path;

use serde_json::json;
use xiuxian_qianji_bpmn_engine::{
    BpmnHostActivityId, BpmnHostProcessId, BpmnTaskOutputBinding, PendingHostWork,
    PendingHostWorkKind,
};
use xiuxian_qianji_control::{
    ActivityId, ActivityTask, ActivityType, AdmittedActivityTaskScheduleRecord, ArtifactId,
    ArtifactKind, ArtifactRef, ControlError, ControlResult, IdempotencyKey, RunId, TaskQueue,
};

/// Metadata key used on Flowhub BPMN service activity tasks.
pub const FLOWHUB_SERVICE_ACTIVITY_METADATA_KEY: &str = "qianji_flowhub_service_task";
/// Metadata schema used on Flowhub BPMN service activity tasks.
pub const FLOWHUB_SERVICE_ACTIVITY_SCHEMA: &str = "xiuxian_qianji.flowhub.service_activity_task.v1";
/// Activity type used for Flowhub BPMN `serviceTask` execution.
pub const FLOWHUB_SERVICE_ACTIVITY_TYPE: &str = "flowhub.service";

const FLOWHUB_SERVICE_QUEUE_PREFIX: &str = "flowhub.";
const FLOWHUB_SERVICE_INPUT_ARTIFACT_KIND: &str = "bpmn.pending_host_work";

/// Borrowed Flowhub scenario id used to derive task queues and claim-check URIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlowhubScenarioIdRef<'a>(&'a str);

impl<'a> FlowhubScenarioIdRef<'a> {
    /// Creates a borrowed Flowhub scenario id reference.
    #[must_use]
    pub const fn new(value: &'a str) -> Self {
        Self(value)
    }

    /// Borrows the serialized scenario id.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

/// Borrowed BPMN workflow instance id used in runtime activity identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QianjiRuntimeBpmnInstanceIdRef<'a>(&'a str);

impl<'a> QianjiRuntimeBpmnInstanceIdRef<'a> {
    /// Creates a borrowed BPMN workflow instance id reference.
    #[must_use]
    pub const fn new(value: &'a str) -> Self {
        Self(value)
    }

    /// Borrows the serialized BPMN workflow instance id.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

/// Runtime event timestamp in Unix milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct QianjiRuntimeInstantMs(u64);

impl QianjiRuntimeInstantMs {
    /// Creates a runtime timestamp from Unix milliseconds.
    #[must_use]
    pub const fn from_millis(value: u64) -> Self {
        Self(value)
    }

    /// Returns the timestamp in Unix milliseconds.
    #[must_use]
    pub const fn as_millis(self) -> u64 {
        self.0
    }
}

/// Input for converting one Flowhub BPMN service boundary into an
/// `ActivityTask` schedule record.
#[derive(Debug, Clone, Copy)]
pub struct FlowhubServiceActivityScheduleInput<'a> {
    /// Owning Qianji control-plane run id.
    pub run_id: &'a RunId,
    /// Schedule timestamp supplied by the caller.
    pub occurred_at_ms: QianjiRuntimeInstantMs,
    /// Flowhub scenario id, for example `agent-coding`.
    pub scenario_id: FlowhubScenarioIdRef<'a>,
    /// BPMN workflow instance id.
    pub instance_id: QianjiRuntimeBpmnInstanceIdRef<'a>,
    /// Source BPMN document path used by the workflow route.
    pub bpmn_source: &'a Path,
    /// Pending BPMN host work currently blocking the workflow.
    pub pending_work: &'a PendingHostWork,
}

/// Builds a durable `ActivityTask` schedule record for one Flowhub BPMN service
/// boundary.
///
/// # Errors
///
/// Returns a control error when the pending work is not a service task, lacks
/// BPMN identity, or contains invalid control-plane identifiers.
pub fn build_flowhub_service_activity_schedule_record(
    input: FlowhubServiceActivityScheduleInput<'_>,
) -> ControlResult<AdmittedActivityTaskScheduleRecord> {
    let process_id = required_process_id(input.pending_work)?;
    let activity_id = required_activity_id(input.pending_work)?;
    require_service_work(input.pending_work)?;

    let control_activity_id = ActivityId::new(flowhub_service_activity_id(
        input.instance_id.as_str(),
        process_id,
        activity_id,
        input.pending_work.token_id,
    ))?;
    let mut task = ActivityTask::new(
        control_activity_id.clone(),
        ActivityType::new(FLOWHUB_SERVICE_ACTIVITY_TYPE)?,
        TaskQueue::new(flowhub_service_task_queue(input.scenario_id.as_str()))?,
        IdempotencyKey::new(flowhub_service_idempotency_key(
            control_activity_id.as_str(),
        ))?,
    )
    .with_input_ref(flowhub_service_input_ref(
        input.scenario_id.as_str(),
        input.instance_id.as_str(),
        process_id,
        activity_id,
        input.pending_work.token_id,
        control_activity_id.as_str(),
    )?);
    task.metadata =
        flowhub_service_metadata(input, process_id, activity_id, control_activity_id.as_str());

    Ok(AdmittedActivityTaskScheduleRecord::run(
        input.run_id.clone(),
        input.occurred_at_ms.as_millis(),
        task,
    ))
}

fn require_service_work(work: &PendingHostWork) -> ControlResult<()> {
    if work.kind == PendingHostWorkKind::Service {
        return Ok(());
    }
    Err(invalid_flowhub_service_task(format!(
        "Flowhub ActivityTask adapter only accepts service work, got `{}`",
        pending_host_work_kind_name(&work.kind)
    )))
}

fn required_process_id(work: &PendingHostWork) -> ControlResult<&str> {
    work.process_id
        .as_ref()
        .map(BpmnHostProcessId::as_str)
        .ok_or_else(missing_process_id_error)
}

fn required_activity_id(work: &PendingHostWork) -> ControlResult<&str> {
    work.activity_id
        .as_ref()
        .map(BpmnHostActivityId::as_str)
        .ok_or_else(missing_activity_id_error)
}

fn flowhub_service_activity_id(
    instance_id: &str,
    process_id: &str,
    activity_id: &str,
    token_id: u64,
) -> String {
    format!("flowhub.{instance_id}.{process_id}.{activity_id}.{token_id}")
}

fn flowhub_service_task_queue(scenario_id: &str) -> String {
    format!("{FLOWHUB_SERVICE_QUEUE_PREFIX}{scenario_id}")
}

fn flowhub_service_idempotency_key(activity_id: &str) -> String {
    format!("idempotency.{activity_id}")
}

fn flowhub_service_input_ref(
    scenario_id: &str,
    instance_id: &str,
    process_id: &str,
    activity_id: &str,
    token_id: u64,
    control_activity_id: &str,
) -> ControlResult<ArtifactRef> {
    Ok(ArtifactRef {
        artifact_id: ArtifactId::new(format!("artifact.{control_activity_id}.input"))?,
        artifact_kind: ArtifactKind::new(FLOWHUB_SERVICE_INPUT_ARTIFACT_KIND)?,
        uri: format!(
            "flowhub://{scenario_id}/instances/{instance_id}/processes/{process_id}/tokens/{token_id}/service-tasks/{activity_id}"
        ),
        content_digest: None,
        metadata: json!({
            "claimCheckKind": "flowhub_pending_host_work",
            "schema": FLOWHUB_SERVICE_ACTIVITY_SCHEMA
        }),
    })
}

fn flowhub_service_metadata(
    input: FlowhubServiceActivityScheduleInput<'_>,
    process_id: &str,
    activity_id: &str,
    control_activity_id: &str,
) -> serde_json::Value {
    json!({
        FLOWHUB_SERVICE_ACTIVITY_METADATA_KEY: {
            "schema": FLOWHUB_SERVICE_ACTIVITY_SCHEMA,
            "scenarioId": input.scenario_id.as_str(),
            "instanceId": input.instance_id.as_str(),
            "bpmnSource": input.bpmn_source.display().to_string(),
            "processId": process_id,
            "activityId": activity_id,
            "controlActivityId": control_activity_id,
            "nodeIndex": input.pending_work.node_index,
            "tokenId": input.pending_work.token_id,
            "workKind": pending_host_work_kind_name(&input.pending_work.kind),
            "workId": input
                .pending_work
                .work_id
                .as_deref(),
            "completion": {
                "httpKind": "service",
                "requiredOutputs": required_output_metadata(input.pending_work)
            }
        }
    })
}

fn required_output_metadata(work: &PendingHostWork) -> Vec<serde_json::Value> {
    work.task_io
        .as_ref()
        .map(|task_io| {
            task_io
                .outputs
                .iter()
                .map(output_binding_metadata)
                .collect()
        })
        .unwrap_or_default()
}

fn output_binding_metadata(output: &BpmnTaskOutputBinding) -> serde_json::Value {
    json!({
        "name": output.name.as_ref(),
        "targetRef": output.target_ref.as_ref(),
        "required": output.required
    })
}

fn missing_process_id_error() -> ControlError {
    invalid_flowhub_service_task("Flowhub service work requires a BPMN process id".to_owned())
}

fn missing_activity_id_error() -> ControlError {
    invalid_flowhub_service_task("Flowhub service work requires a BPMN activity id".to_owned())
}

fn pending_host_work_kind_name(kind: &PendingHostWorkKind) -> &'static str {
    match kind {
        PendingHostWorkKind::Send => "send",
        PendingHostWorkKind::Service => "service",
        PendingHostWorkKind::Script => "script",
        PendingHostWorkKind::User => "user",
        PendingHostWorkKind::Manual => "manual",
        PendingHostWorkKind::BusinessRule => "business_rule",
    }
}

fn invalid_flowhub_service_task(message: String) -> ControlError {
    ControlError::InvalidEventSequence { message }
}
