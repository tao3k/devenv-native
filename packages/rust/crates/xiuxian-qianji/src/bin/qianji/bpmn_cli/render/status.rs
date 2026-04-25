use std::fmt::Write as _;

use crate::bpmn_cli::deps::{
    BpmnInstanceState, BpmnPackage, BpmnProcessSpec, QianjiBpmnWorkflowStatusReport,
};
use crate::bpmn_cli::types::{BpmnCliOutput, BpmnStatusCliCommand};

use super::support::{
    bpmn_checkpoint_backend_label, bpmn_checkpoint_backend_selection_label, bpmn_event_kind_label,
    bpmn_lifecycle_label, bpmn_node_id_label, bpmn_node_kind_label,
    bpmn_pending_host_work_kind_label, bpmn_suspend_reason_label, bpmn_timer_spec_label,
    bpmn_wait_kind_label, node_runtime_status_label,
};

pub(crate) fn render_bpmn_status_output(
    command: &BpmnStatusCliCommand,
    report: &QianjiBpmnWorkflowStatusReport,
    package: Option<&BpmnPackage>,
) -> BpmnCliOutput {
    let variables = serde_json::to_string_pretty(&report.instance.variables)
        .unwrap_or_else(|error| format!("{{\"serialization_error\":\"{error}\"}}"));
    let process = package
        .and_then(|package| package.find_process(report.instance.process.process_id.as_ref()));
    let mut rendered = format!(
        "# BPMN Status\n\nInstance: {}\nProcess: {}\nPackage: {}\nLifecycle: {}\nCheckpoint backend: {}\nCheckpoint status: loaded\nCheckpoint sequence: {}\nState sequence: {}\nUpdated at (unix ms): {}\nActive tokens: {}\nPending host work: {}\nWait registrations: {}\nCall stack depth: {}\n",
        command.instance_id,
        report.instance.process.process_id,
        report.instance.process.package_id,
        bpmn_lifecycle_label(&report.instance.lifecycle),
        bpmn_checkpoint_backend_label(&report.checkpoint_store),
        report.checkpoint_sequence,
        report.instance.sequence,
        report.instance.updated_at_ms,
        report.instance.active_tokens.len(),
        report.instance.pending_host_work.len(),
        report.instance.waits.len(),
        report.instance.call_stack.len(),
    );

    if let Some(reason) = report.instance.suspend_reason.as_ref() {
        let _ = writeln!(
            rendered,
            "\nSuspend reason: {}",
            bpmn_suspend_reason_label(reason)
        );
    }

    append_bpmn_status_active_tokens(&mut rendered, &report.instance, process);
    append_bpmn_status_pending_host_work(&mut rendered, &report.instance, process);
    append_bpmn_status_wait_registrations(&mut rendered, &report.instance, process);
    append_bpmn_status_graph_snapshot(&mut rendered, &report.instance, process);

    let _ = writeln!(rendered, "\n## Variables");
    let _ = writeln!(rendered, "```json");
    let _ = writeln!(rendered, "{variables}");
    let _ = writeln!(rendered, "```");

    BpmnCliOutput {
        rendered,
        exit_code: 0,
    }
}

pub(crate) fn render_bpmn_status_missing_output(command: &BpmnStatusCliCommand) -> BpmnCliOutput {
    BpmnCliOutput {
        rendered: format!(
            "# BPMN Status\n\nInstance: {}\nCheckpoint backend: {}\nCheckpoint status: missing\n",
            command.instance_id,
            bpmn_checkpoint_backend_selection_label(&command.checkpoint_backend),
        ),
        exit_code: 1,
    }
}

fn append_bpmn_status_active_tokens(
    rendered: &mut String,
    instance: &BpmnInstanceState,
    process: Option<&BpmnProcessSpec>,
) {
    if instance.active_tokens.is_empty() {
        return;
    }

    let _ = writeln!(rendered, "\n## Active Tokens");
    for token in &instance.active_tokens {
        let mut line = format!(
            "- token#{} | node_index={}",
            token.token_id, token.node_index
        );
        append_bpmn_status_node_context(&mut line, process, token.node_index);
        if let Some(incoming_edge_index) = token.incoming_edge_index {
            let _ = write!(line, " | incoming_edge={incoming_edge_index}");
        }
        if let Some(hint) = token.inclusive_join_hint.as_ref() {
            let _ = write!(
                line,
                " | inclusive_join={} | activation={} | expected_arrivals={}",
                hint.join_node_index, hint.activation_id, hint.expected_arrivals
            );
        }
        let _ = writeln!(rendered, "{line}");
    }
}

fn append_bpmn_status_pending_host_work(
    rendered: &mut String,
    instance: &BpmnInstanceState,
    process: Option<&BpmnProcessSpec>,
) {
    if instance.pending_host_work.is_empty() {
        return;
    }

    let _ = writeln!(rendered, "\n## Pending Host Work");
    for work in &instance.pending_host_work {
        let mut line = format!(
            "- token#{} | node_index={} | kind={}",
            work.token_id,
            work.node_index,
            bpmn_pending_host_work_kind_label(&work.kind)
        );
        append_bpmn_status_node_context(&mut line, process, work.node_index);
        if let Some(process_id) = work.process_id.as_ref() {
            let _ = write!(line, " | process={process_id}");
        }
        if let Some(work_id) = work.work_id.as_ref() {
            let _ = write!(line, " | work_id={work_id}");
        }
        if let Some(event_reference) = work.event_reference.as_ref() {
            let _ = write!(line, " | ref={event_reference}");
        }
        if let Some(event_name) = work.event_name.as_ref() {
            let _ = write!(line, " | name={event_name}");
        }
        if let Some(decision) = work.decision.as_ref() {
            let _ = write!(line, " | decision={}", decision.decision_id);
        }
        if let Some(script_format) = work.script_format.as_ref() {
            let _ = write!(line, " | script_format={script_format}");
        }
        let _ = writeln!(rendered, "{line}");
    }
}

fn append_bpmn_status_wait_registrations(
    rendered: &mut String,
    instance: &BpmnInstanceState,
    process: Option<&BpmnProcessSpec>,
) {
    if instance.waits.is_empty() {
        return;
    }

    let _ = writeln!(rendered, "\n## Wait Registrations");
    if let Some(competition) = instance.event_competition.as_ref() {
        let _ = writeln!(
            rendered,
            "Competition gateway node_index: {}",
            competition.gateway_node_index
        );
    }

    for wait in &instance.waits {
        let mut line = format!(
            "- node_index={} | kind={}",
            wait.node_index,
            bpmn_wait_kind_label(&wait.kind)
        );
        append_bpmn_status_node_context(&mut line, process, wait.node_index);
        if let Some(event_kind) = wait.event_kind.as_ref() {
            let _ = write!(line, " | event={}", bpmn_event_kind_label(event_kind));
        }
        if let Some(reference) = wait.event_reference.as_ref() {
            let _ = write!(line, " | ref={reference}");
        }
        if let Some(name) = wait.event_name.as_ref() {
            let _ = write!(line, " | name={name}");
        }
        if let Some(timer) = wait.timer.as_ref() {
            let _ = write!(line, " | timer={}", bpmn_timer_spec_label(timer));
        }
        if let Some(blocking_node_index) = wait.blocking_node_index {
            let _ = write!(line, " | blocking_node_index={blocking_node_index}");
        }
        if let Some(correlation_key) = wait.correlation_key.as_ref() {
            let _ = write!(line, " | correlation={correlation_key}");
        }
        let _ = writeln!(rendered, "{line}");
    }
}

fn append_bpmn_status_node_context(
    line: &mut String,
    process: Option<&BpmnProcessSpec>,
    node_index: u32,
) {
    let Some(process) = process else {
        return;
    };
    if let Some(node) = process.nodes.get(node_index as usize) {
        let _ = write!(
            line,
            " | node_id={} | node_kind={}",
            node.bpmn_id,
            bpmn_node_kind_label(&node.kind)
        );
        return;
    }
    let _ = write!(
        line,
        " | node_id={}",
        bpmn_node_id_label(process, node_index)
    );
}

fn append_bpmn_status_graph_snapshot(
    rendered: &mut String,
    instance: &BpmnInstanceState,
    process: Option<&BpmnProcessSpec>,
) {
    let Some(process) = process else {
        return;
    };
    let values = process
        .nodes
        .iter()
        .enumerate()
        .map(|(node_index, node)| {
            let status = instance
                .node_states
                .get(node_index)
                .map(|state| node_runtime_status_label(&state.status))
                .unwrap_or("unknown");
            serde_json::json!({
                "node_id": node.bpmn_id.as_ref(),
                "node_index": node.index,
                "node_kind": bpmn_node_kind_label(&node.kind),
                "status": status,
            })
        })
        .collect::<Vec<_>>();
    let snapshot = serde_json::to_string_pretty(&values)
        .unwrap_or_else(|error| format!("[{{\"serialization_error\":\"{error}\"}}]"));
    let _ = writeln!(rendered, "\n## Graph Snapshot");
    let _ = writeln!(rendered, "```json");
    let _ = writeln!(rendered, "{snapshot}");
    let _ = writeln!(rendered, "```");
}
