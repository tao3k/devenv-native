use std::fmt::Write as _;

use crate::bpmn_cli::deps::{
    BpmnProcessSpec, PendingHostWorkRequest, QianjiBpmnSession, build_pending_host_work_requests,
};

use crate::bpmn_cli::render::support::{bpmn_node_id_label, bpmn_pending_host_work_kind_label};

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
            let _ = write!(line, " | process={process_id}");
        }
        if let Some(work_id) = work.work_id.as_ref() {
            let _ = write!(line, " | work_id={work_id}");
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
            "node_id": pending_host_work_request_node_id(process, request.node_index),
            "node_index": request.node_index,
            "token_id": request.token_id,
            "variables": request.variables,
            "message_reference": request.message_reference,
            "message_name": request.message_name,
        }),
        PendingHostWorkRequest::Service(request) => serde_json::json!({
            "kind": "service",
            "node_id": pending_host_work_request_node_id(process, request.node_index),
            "node_index": request.node_index,
            "token_id": request.token_id,
            "variables": request.variables,
            "repeat": request.repeat,
        }),
        PendingHostWorkRequest::Script(request) => serde_json::json!({
            "kind": "script",
            "node_id": pending_host_work_request_node_id(process, request.node_index),
            "node_index": request.node_index,
            "token_id": request.token_id,
            "variables": request.variables,
            "repeat": request.repeat,
            "script_format": request.script_format,
            "script_body": request.script_body,
        }),
        PendingHostWorkRequest::User(request) => serde_json::json!({
            "kind": "user",
            "node_id": pending_host_work_request_node_id(process, request.node_index),
            "node_index": request.node_index,
            "token_id": request.token_id,
            "variables": request.variables,
            "repeat": request.repeat,
        }),
        PendingHostWorkRequest::Manual(request) => serde_json::json!({
            "kind": "manual",
            "node_id": pending_host_work_request_node_id(process, request.node_index),
            "node_index": request.node_index,
            "token_id": request.token_id,
            "variables": request.variables,
            "repeat": request.repeat,
        }),
        PendingHostWorkRequest::BusinessRule(request) => serde_json::json!({
            "kind": "business_rule",
            "node_id": pending_host_work_request_node_id(process, request.node_index),
            "node_index": request.node_index,
            "token_id": request.token_id,
            "evaluation": request.evaluation,
            "repeat": request.repeat,
        }),
    }
}

fn pending_host_work_request_node_id(process: &BpmnProcessSpec, node_index: u32) -> String {
    bpmn_node_id_label(process, node_index)
}
