use std::fmt::Write as _;

use crate::qianji_cli::bpmn_cli::deps::{
    BpmnProcessSpec, PendingHostWorkRequest, QianjiBpmnSession, build_pending_host_work_requests,
};

use crate::qianji_cli::bpmn_cli::render::support::{
    bpmn_human_task_assignment_label, bpmn_human_task_form_label, bpmn_lane_membership_label,
    bpmn_node_id_label, bpmn_pending_host_work_kind_label,
};

pub(super) fn append_bpmn_pending_host_work(rendered: &mut String, session: &QianjiBpmnSession) {
    if session.instance().pending_host_work.is_empty() {
        return;
    }

    let Some(process) = session
        .package()
        .find_process(session.instance().process.process_id.as_ref())
    else {
        return;
    };

    let _ = writeln!(rendered, "\n## Pending Host Work");
    for work in &session.instance().pending_host_work {
        let node_id = process.nodes.get(work.node_index as usize).map_or_else(
            || format!("node#{}", work.node_index),
            |node| node.bpmn_id.to_string(),
        );
        let mut line = format!(
            "- {node_id} | token#{} | kind={}",
            work.token_id,
            bpmn_pending_host_work_kind_label(&work.kind)
        );
        if let Some(process_id) = work.process_id.as_ref() {
            let _ = write!(line, " | process={}", process_id.as_str());
        }
        if let Some(activity_id) = work.activity_id.as_ref() {
            let _ = write!(line, " | activity={}", activity_id.as_str());
        }
        if let Some(work_id) = work.work_id.as_ref() {
            let _ = write!(line, " | work_id={}", work_id.as_str());
        }
        if let Some(claim) = work.claim.as_ref() {
            let _ = write!(line, " | claim={}", claim.claimant);
        }
        if let Some(form) = work.human_task_form.as_ref() {
            let _ = write!(line, " | form={}", bpmn_human_task_form_label(form));
        }
        if let Some(assignment) = work.human_task_assignment.as_ref() {
            let label = bpmn_human_task_assignment_label(assignment);
            if !label.is_empty() {
                let _ = write!(line, " | assignment={label}");
            }
        }
        if let Some(lane) = work.lane.as_ref() {
            let _ = write!(line, " | lane={}", bpmn_lane_membership_label(lane));
        }
        let _ = writeln!(rendered, "{line}");
    }
}

pub(crate) fn render_bpmn_pending_host_work_stream_lines(
    session: &QianjiBpmnSession,
) -> Vec<String> {
    if session.instance().pending_host_work.is_empty() {
        return Vec::new();
    }

    let Some(process) = session
        .package()
        .find_process(session.instance().process.process_id.as_ref())
    else {
        return Vec::new();
    };

    let Ok(requests) = build_pending_host_work_requests(session.instance()) else {
        return Vec::new();
    };

    requests
        .into_iter()
        .filter_map(|request| {
            let value = pending_host_work_request_stream_value(process, request);
            serde_json::to_string(&value)
                .ok()
                .map(|payload| format!("@@QIANJI_HOST_WORK {payload}"))
        })
        .collect()
}

fn pending_host_work_request_stream_value(
    process: &BpmnProcessSpec,
    request: PendingHostWorkRequest,
) -> serde_json::Value {
    match request {
        PendingHostWorkRequest::Send(request) => serde_json::json!({
            "kind": "send",
            "instance_id": request.instance_id,
            "node_id": pending_host_work_request_node_id(process, request.node_index),
            "node_index": request.node_index,
            "token_id": request.token_id,
            "variables": request.variables,
            "inputs": request.inputs,
            "output_bindings": request.output_bindings,
            "message_reference": request.message_reference,
            "message_name": request.message_name,
        }),
        PendingHostWorkRequest::Service(request) => serde_json::json!({
            "kind": "service",
            "instance_id": request.instance_id,
            "node_id": pending_host_work_request_node_id(process, request.node_index),
            "node_index": request.node_index,
            "token_id": request.token_id,
            "variables": request.variables,
            "inputs": request.inputs,
            "output_bindings": request.output_bindings,
            "repeat": request.repeat,
        }),
        PendingHostWorkRequest::Script(request) => serde_json::json!({
            "kind": "script",
            "instance_id": request.instance_id,
            "node_id": pending_host_work_request_node_id(process, request.node_index),
            "node_index": request.node_index,
            "token_id": request.token_id,
            "variables": request.variables,
            "inputs": request.inputs,
            "output_bindings": request.output_bindings,
            "repeat": request.repeat,
            "script_format": request.script_format,
            "script_body": request.script_body,
        }),
        PendingHostWorkRequest::User(request) => serde_json::json!({
            "kind": "user",
            "instance_id": request.instance_id,
            "process_id": request.process_id,
            "activity_id": request.activity_id,
            "node_id": pending_host_work_request_node_id(process, request.node_index),
            "node_index": request.node_index,
            "token_id": request.token_id,
            "variables": request.variables,
            "inputs": request.inputs,
            "output_bindings": request.output_bindings,
            "repeat": request.repeat,
            "form": request.form,
            "assignment": request.assignment,
            "lane": request.lane,
            "claim": request.claim,
        }),
        PendingHostWorkRequest::Manual(request) => serde_json::json!({
            "kind": "manual",
            "instance_id": request.instance_id,
            "process_id": request.process_id,
            "activity_id": request.activity_id,
            "node_id": pending_host_work_request_node_id(process, request.node_index),
            "node_index": request.node_index,
            "token_id": request.token_id,
            "variables": request.variables,
            "inputs": request.inputs,
            "output_bindings": request.output_bindings,
            "repeat": request.repeat,
            "form": request.form,
            "assignment": request.assignment,
            "lane": request.lane,
            "claim": request.claim,
        }),
        PendingHostWorkRequest::BusinessRule(request) => serde_json::json!({
            "kind": "business_rule",
            "instance_id": request.instance_id,
            "node_id": pending_host_work_request_node_id(process, request.node_index),
            "node_index": request.node_index,
            "token_id": request.token_id,
            "evaluation": request.evaluation,
            "inputs": request.inputs,
            "output_bindings": request.output_bindings,
            "repeat": request.repeat,
        }),
    }
}

fn pending_host_work_request_node_id(process: &BpmnProcessSpec, node_index: u32) -> String {
    bpmn_node_id_label(process, node_index)
}
