//! Durable control trace projection for qianji-server BPMN HTTP runs.

use serde_json::{Map, Value, json};
use std::collections::BTreeSet;
use std::path::Path;
use xiuxian_qianji_bpmn_engine::{
    BpmnExecutionTraceEvent, BpmnExecutionTraceEventKind, BpmnNodeIndex, BpmnNodeSpec,
    BpmnProcessSpec, InstanceLifecycle, NodeRuntimeStatus,
};
use xiuxian_qianji_control::{
    ControlEventKind, ControlLedger, RunAdmittedJournalRecord, RunCreatedJournalRecord, RunId,
    RunPlanRecordedJournalRecord, RunTerminalJournalRecord, StepCreatedJournalRecord,
    StepFailureJournalInput, StepId, StepStartedJournalRecord, StepTerminalJournalRecord,
    StepToolCallJournalRecord, record_control_event_batch,
};

use super::activity_evidence::now_unix_ms;
use super::error_api::QianjiBpmnWorkflowHttpError;
use crate::bpmn::session::QianjiBpmnSession;

const BPMN_CONTROL_TRACE_SCHEMA: &str = "xiuxian_qianji.bpmn.control_trace.v1";
const BPMN_CONTROL_TRACE_TOOL: &str = "qianji.bpmn.trace";

pub(super) fn record_bpmn_control_trace(
    ledger: Option<&dyn ControlLedger>,
    session: &QianjiBpmnSession,
    bpmn_source: Option<&Path>,
) -> Result<(), QianjiBpmnWorkflowHttpError> {
    let Some(ledger) = ledger else {
        return Ok(());
    };
    let run_id = bpmn_control_run_id(session)?;
    let existing_records = ledger.load_events(&run_id).map_err(control_trace_error)?;
    let mut existing_trace_sequences = BTreeSet::new();
    let mut declared_steps = BTreeSet::new();
    for record in &existing_records {
        if let Some(step_id) = &record.event.step_id
            && matches!(record.event.kind, ControlEventKind::StepCreated { .. })
        {
            declared_steps.insert(step_id.as_str().to_owned());
        }
        if let ControlEventKind::ToolCallRecorded { metadata, .. } = &record.event.kind
            && metadata
                .get("schema")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|schema| schema == BPMN_CONTROL_TRACE_SCHEMA)
            && let Some(sequence) = metadata
                .get("traceSequence")
                .and_then(serde_json::Value::as_u64)
        {
            existing_trace_sequences.insert(sequence);
        }
    }

    let now_ms = now_unix_ms();
    let mut events = Vec::new();
    if existing_records.is_empty() {
        let metadata = run_created_metadata(session, bpmn_source);
        events.push(
            RunCreatedJournalRecord::new(
                run_id.clone(),
                format!("BPMN workflow {}", session.instance().instance_id.as_ref()),
                now_ms,
            )
            .with_metadata(metadata)
            .into_event(),
        );
        events.push(RunAdmittedJournalRecord::new(run_id.clone(), now_ms).into_event());
        events.push(
            RunPlanRecordedJournalRecord::new(
                run_id.clone(),
                format!(
                    "BPMN process {} execution trace",
                    session.instance().process.process_id.as_ref()
                ),
                now_ms,
            )
            .into_event(),
        );
    }

    let mut appended_trace_event = false;
    for event in &session.instance().trace {
        if existing_trace_sequences.contains(&event.sequence) {
            continue;
        }
        let event_count_before = events.len();
        append_trace_event_projection(
            session,
            &run_id,
            event,
            now_ms.saturating_add(event.sequence),
            &mut declared_steps,
            &mut events,
        )?;
        appended_trace_event |= events.len() > event_count_before;
    }
    if existing_records.is_empty() || appended_trace_event {
        append_run_status_projection(session, &run_id, now_ms, &mut events);
    }

    if events.is_empty() {
        return Ok(());
    }
    record_control_event_batch(ledger, events)
        .map(|_| ())
        .map_err(control_trace_error)
}

fn run_created_metadata(session: &QianjiBpmnSession, bpmn_source: Option<&Path>) -> Value {
    let mut metadata = Map::new();
    metadata.insert("schema".to_owned(), json!(BPMN_CONTROL_TRACE_SCHEMA));
    metadata.insert("source".to_owned(), json!("qianji-server"));
    metadata.insert(
        "instanceId".to_owned(),
        json!(session.instance().instance_id.as_ref()),
    );
    metadata.insert(
        "processId".to_owned(),
        json!(session.instance().process.process_id.as_ref()),
    );
    if let Some(path) = bpmn_source {
        let source_ref = path.display().to_string();
        metadata.insert("bpmnSourceKind".to_owned(), json!("filesystem_path"));
        metadata.insert("bpmnSourceRef".to_owned(), json!(source_ref.clone()));
        metadata.insert("bpmn_source_ref".to_owned(), json!(source_ref));
    }
    Value::Object(metadata)
}

fn append_trace_event_projection(
    session: &QianjiBpmnSession,
    run_id: &RunId,
    event: &BpmnExecutionTraceEvent,
    occurred_at_ms: u64,
    declared_steps: &mut BTreeSet<String>,
    events: &mut Vec<xiuxian_qianji_control::ControlEvent>,
) -> Result<(), QianjiBpmnWorkflowHttpError> {
    let BpmnExecutionTraceEventKind::NodeStatus = event.kind else {
        return Ok(());
    };
    let Some(node_index) = event.node_index else {
        return Ok(());
    };
    let process = session
        .package()
        .find_process_position(event.process.process_id.as_ref())
        .map(|(_, process)| process);
    let node = node_by_index(process, node_index);
    let step_id = StepId::new(node_id(node, node_index)).map_err(control_trace_error)?;
    append_step_created_if_needed(
        run_id,
        &step_id,
        node,
        node_index,
        occurred_at_ms,
        declared_steps,
        events,
    );
    let metadata = trace_event_metadata(session, process, node, event, node_index);
    match event.status {
        Some(NodeRuntimeStatus::Queued) => {
            events.push(xiuxian_qianji_control::ControlEvent::step(
                run_id.clone(),
                step_id,
                occurred_at_ms,
                ControlEventKind::StepQueued,
            ));
        }
        Some(NodeRuntimeStatus::Executing) => {
            events.push(
                StepStartedJournalRecord::new(run_id.clone(), step_id, occurred_at_ms).into_event(),
            );
        }
        Some(NodeRuntimeStatus::Completed) => {
            append_trace_tool_call(run_id, &step_id, occurred_at_ms, metadata, events);
            events.push(
                StepTerminalJournalRecord::succeeded(run_id.clone(), step_id, occurred_at_ms)
                    .into_event(),
            );
        }
        Some(NodeRuntimeStatus::Cancelled) => {
            append_trace_tool_call(run_id, &step_id, occurred_at_ms, metadata, events);
            events.push(
                StepTerminalJournalRecord::cancelled(
                    run_id.clone(),
                    step_id,
                    "BPMN node cancelled",
                    occurred_at_ms,
                )
                .into_event(),
            );
        }
        Some(NodeRuntimeStatus::Failed) => {
            append_trace_tool_call(run_id, &step_id, occurred_at_ms, metadata, events);
            events.push(
                StepTerminalJournalRecord::failed(
                    run_id.clone(),
                    step_id,
                    StepFailureJournalInput::new("bpmn_node_failed", "BPMN node failed", false),
                    occurred_at_ms,
                )
                .into_event(),
            );
        }
        Some(NodeRuntimeStatus::Idle) | None => {}
    }
    Ok(())
}

fn append_step_created_if_needed(
    run_id: &RunId,
    step_id: &StepId,
    node: Option<&BpmnNodeSpec>,
    node_index: BpmnNodeIndex,
    occurred_at_ms: u64,
    declared_steps: &mut BTreeSet<String>,
    events: &mut Vec<xiuxian_qianji_control::ControlEvent>,
) {
    if declared_steps.insert(step_id.as_str().to_owned()) {
        events.push(
            StepCreatedJournalRecord::new(
                run_id.clone(),
                step_id.clone(),
                step_title(node, node_index),
                occurred_at_ms,
            )
            .into_event(),
        );
    }
}

fn append_trace_tool_call(
    run_id: &RunId,
    step_id: &StepId,
    occurred_at_ms: u64,
    metadata: serde_json::Value,
    events: &mut Vec<xiuxian_qianji_control::ControlEvent>,
) {
    events.push(
        StepToolCallJournalRecord::new(
            run_id.clone(),
            step_id.clone(),
            BPMN_CONTROL_TRACE_TOOL,
            occurred_at_ms,
        )
        .with_metadata(metadata)
        .into_event(),
    );
}

fn append_run_status_projection(
    session: &QianjiBpmnSession,
    run_id: &RunId,
    occurred_at_ms: u64,
    events: &mut Vec<xiuxian_qianji_control::ControlEvent>,
) {
    match session.instance().lifecycle {
        InstanceLifecycle::Completed => {
            events.push(
                RunTerminalJournalRecord::completed(run_id.clone(), occurred_at_ms).into_event(),
            );
        }
        InstanceLifecycle::Failed => {
            events.push(
                RunTerminalJournalRecord::failed(
                    run_id.clone(),
                    "BPMN workflow failed",
                    occurred_at_ms,
                )
                .into_event(),
            );
        }
        InstanceLifecycle::Waiting => {
            events.push(
                RunTerminalJournalRecord::blocked(
                    run_id.clone(),
                    "BPMN workflow waiting",
                    occurred_at_ms,
                )
                .into_event(),
            );
        }
        InstanceLifecycle::Suspended => {
            events.push(
                RunTerminalJournalRecord::blocked(
                    run_id.clone(),
                    "BPMN workflow suspended",
                    occurred_at_ms,
                )
                .into_event(),
            );
        }
        InstanceLifecycle::Ready | InstanceLifecycle::Running => {}
    }
}

fn trace_event_metadata(
    session: &QianjiBpmnSession,
    process: Option<&BpmnProcessSpec>,
    node: Option<&BpmnNodeSpec>,
    event: &BpmnExecutionTraceEvent,
    node_index: BpmnNodeIndex,
) -> serde_json::Value {
    let element_id = node_id(node, node_index);
    json!({
        "schema": BPMN_CONTROL_TRACE_SCHEMA,
        "source": "qianji-server",
        "eventKind": "node_status",
        "traceSequence": event.sequence,
        "instanceId": session.instance().instance_id.as_ref(),
        "processId": event.process.process_id.as_ref(),
        "bpmnElementId": element_id,
        "bpmn_element_id": element_id,
        "element_id": element_id,
        "node_id": element_id,
        "activity_id": element_id,
        "nodeIndex": node_index,
        "nodeKind": node.and_then(|node| serde_json::to_value(&node.kind).ok()),
        "runtimeStatus": event.status.as_ref().and_then(|status| serde_json::to_value(status).ok()),
        "processSpecDigest": process.map(|process| process.key.spec_digest_hex.to_string())
    })
}

fn bpmn_control_run_id(session: &QianjiBpmnSession) -> Result<RunId, QianjiBpmnWorkflowHttpError> {
    RunId::new(format!(
        "bpmn.workflow.{}",
        session.instance().instance_id.as_ref()
    ))
    .map_err(control_trace_error)
}

fn node_by_index(
    process: Option<&BpmnProcessSpec>,
    node_index: BpmnNodeIndex,
) -> Option<&BpmnNodeSpec> {
    process.and_then(|process| process.nodes.get(node_index as usize))
}

fn node_id(node: Option<&BpmnNodeSpec>, node_index: BpmnNodeIndex) -> String {
    node.map_or_else(|| node_index.to_string(), |node| node.bpmn_id.to_string())
}

fn step_title(node: Option<&BpmnNodeSpec>, node_index: BpmnNodeIndex) -> String {
    node.map_or_else(
        || format!("BPMN node {node_index}"),
        |node| format!("BPMN {} ({:?})", node.bpmn_id, node.kind),
    )
}

fn control_trace_error(error: impl std::fmt::Display) -> QianjiBpmnWorkflowHttpError {
    QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string())
}
