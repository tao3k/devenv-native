//! Generic BPMN host-work activity schedule construction.

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

use crate::flowhub::{QianjiRuntimeBpmnInstanceIdRef, QianjiRuntimeInstantMs};

/// Metadata key used on generic BPMN host-work activity tasks.
pub const BPMN_HOST_WORK_ACTIVITY_METADATA_KEY: &str = "qianji_bpmn_host_work_activity";
/// Metadata schema used on generic BPMN host-work activity tasks.
pub const BPMN_HOST_WORK_ACTIVITY_SCHEMA: &str = "xiuxian_qianji.bpmn.host_work_activity.v1";
/// Activity type used for generic BPMN host-work execution evidence.
pub const BPMN_HOST_WORK_ACTIVITY_TYPE: &str = "bpmn.host_work";

const BPMN_HOST_WORK_INPUT_ARTIFACT_KIND: &str = "bpmn.pending_host_work";
const BPMN_HOST_WORK_QUEUE_PREFIX: &str = "bpmn.host_work.";

/// Input for converting one BPMN pending host-work boundary into an
/// `ActivityTask` schedule record.
#[derive(Debug, Clone, Copy)]
pub struct BpmnHostWorkActivityScheduleInput<'a> {
    /// Owning Qianji control-plane run id.
    pub run_id: &'a RunId,
    /// Schedule timestamp supplied by the caller.
    pub occurred_at_ms: QianjiRuntimeInstantMs,
    /// BPMN workflow instance id.
    pub instance_id: QianjiRuntimeBpmnInstanceIdRef<'a>,
    /// Source BPMN document path used by the workflow route.
    pub bpmn_source: &'a Path,
    /// Pending BPMN host work currently blocking the workflow.
    pub pending_work: &'a PendingHostWork,
}

/// Builds a durable `ActivityTask` schedule record for one generic BPMN
/// pending host-work item.
///
/// # Errors
///
/// Returns a control error when the pending work lacks BPMN identity or
/// contains invalid control-plane identifiers.
pub fn build_bpmn_host_work_activity_schedule_record(
    input: BpmnHostWorkActivityScheduleInput<'_>,
) -> ControlResult<AdmittedActivityTaskScheduleRecord> {
    let process_id = required_process_id(input.pending_work)?;
    let activity_id = required_activity_id(input.pending_work)?;
    let control_activity_id = ActivityId::new(bpmn_host_work_activity_id(
        input.instance_id.as_str(),
        process_id,
        activity_id,
        input.pending_work.token_id,
    ))?;
    let mut task = ActivityTask::new(
        control_activity_id.clone(),
        ActivityType::new(BPMN_HOST_WORK_ACTIVITY_TYPE)?,
        TaskQueue::new(bpmn_host_work_task_queue(&input.pending_work.kind))?,
        IdempotencyKey::new(bpmn_host_work_idempotency_key(control_activity_id.as_str()))?,
    )
    .with_input_ref(bpmn_host_work_input_ref(
        input.instance_id.as_str(),
        process_id,
        activity_id,
        input.pending_work.token_id,
        control_activity_id.as_str(),
    )?);
    task.metadata =
        bpmn_host_work_metadata(input, process_id, activity_id, control_activity_id.as_str());

    Ok(AdmittedActivityTaskScheduleRecord::run(
        input.run_id.clone(),
        input.occurred_at_ms.as_millis(),
        task,
    ))
}

fn required_process_id(work: &PendingHostWork) -> ControlResult<&str> {
    work.process_id
        .as_ref()
        .map(BpmnHostProcessId::as_str)
        .ok_or_else(|| invalid_bpmn_host_work("BPMN host work requires a process id"))
}

fn required_activity_id(work: &PendingHostWork) -> ControlResult<&str> {
    work.activity_id
        .as_ref()
        .map(BpmnHostActivityId::as_str)
        .ok_or_else(|| invalid_bpmn_host_work("BPMN host work requires an activity id"))
}

fn bpmn_host_work_activity_id(
    instance_id: &str,
    process_id: &str,
    activity_id: &str,
    token_id: u64,
) -> String {
    format!("bpmn.{instance_id}.{process_id}.{activity_id}.{token_id}")
}

fn bpmn_host_work_task_queue(kind: &PendingHostWorkKind) -> String {
    format!(
        "{BPMN_HOST_WORK_QUEUE_PREFIX}{}",
        pending_host_work_kind_name(kind)
    )
}

fn bpmn_host_work_idempotency_key(activity_id: &str) -> String {
    format!("idempotency.{activity_id}")
}

fn bpmn_host_work_input_ref(
    instance_id: &str,
    process_id: &str,
    activity_id: &str,
    token_id: u64,
    control_activity_id: &str,
) -> ControlResult<ArtifactRef> {
    Ok(ArtifactRef {
        artifact_id: ArtifactId::new(format!("artifact.{control_activity_id}.input"))?,
        artifact_kind: ArtifactKind::new(BPMN_HOST_WORK_INPUT_ARTIFACT_KIND)?,
        uri: format!(
            "bpmn://instances/{instance_id}/processes/{process_id}/tokens/{token_id}/host-work/{activity_id}"
        ),
        content_digest: None,
        metadata: json!({
            "claimCheckKind": "bpmn_pending_host_work",
            "schema": BPMN_HOST_WORK_ACTIVITY_SCHEMA
        }),
    })
}

fn bpmn_host_work_metadata(
    input: BpmnHostWorkActivityScheduleInput<'_>,
    process_id: &str,
    activity_id: &str,
    control_activity_id: &str,
) -> serde_json::Value {
    json!({
        BPMN_HOST_WORK_ACTIVITY_METADATA_KEY: {
            "schema": BPMN_HOST_WORK_ACTIVITY_SCHEMA,
            "instanceId": input.instance_id.as_str(),
            "bpmnSource": input.bpmn_source.display().to_string(),
            "processId": process_id,
            "activityId": activity_id,
            "controlActivityId": control_activity_id,
            "nodeIndex": input.pending_work.node_index,
            "tokenId": input.pending_work.token_id,
            "workKind": pending_host_work_kind_name(&input.pending_work.kind),
            "workId": input.pending_work.work_id.as_deref(),
            "requiredOutputs": required_output_metadata(input.pending_work)
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

fn pending_host_work_kind_name(kind: &PendingHostWorkKind) -> &'static str {
    match kind {
        PendingHostWorkKind::Task => "task",
        PendingHostWorkKind::Send => "send",
        PendingHostWorkKind::Service => "service",
        PendingHostWorkKind::Script => "script",
        PendingHostWorkKind::User => "user",
        PendingHostWorkKind::Manual => "manual",
        PendingHostWorkKind::BusinessRule => "business_rule",
    }
}

fn invalid_bpmn_host_work(message: &str) -> ControlError {
    ControlError::InvalidEventSequence {
        message: message.to_owned(),
    }
}
