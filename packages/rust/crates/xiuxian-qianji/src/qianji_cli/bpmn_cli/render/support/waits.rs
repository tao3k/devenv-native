use std::fmt::Write as _;

use crate::qianji_cli::bpmn_cli::deps::{BpmnInstanceState, BpmnPackage, BpmnProcessSpec};

use super::labels::{
    bpmn_event_kind_label, bpmn_node_id_label, bpmn_timer_spec_label, bpmn_wait_kind_label,
};

pub(in crate::qianji_cli::bpmn_cli::render) fn append_bpmn_wait_registrations(
    rendered: &mut String,
    package: &BpmnPackage,
    instance: &BpmnInstanceState,
) {
    if instance.waits.is_empty() {
        return;
    }

    let Some(process) = package.find_process(instance.process.process_id.as_ref()) else {
        return;
    };
    let mut wait_lines = instance
        .waits
        .iter()
        .map(|wait| {
            let wait_id = render_bpmn_wait_node_id(process, wait.node_index);
            let mut line = format!("- {wait_id} | kind={}", bpmn_wait_kind_label(&wait.kind));
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
                let _ = write!(
                    line,
                    " | blocking={}",
                    render_bpmn_wait_node_id(process, blocking_node_index)
                );
            }
            if let Some(deduplication_key) = wait.deduplication_key.as_ref() {
                let _ = write!(line, " | dedupe={deduplication_key}");
            }
            (wait_id, line)
        })
        .collect::<Vec<_>>();
    wait_lines.sort_by(|left, right| left.0.cmp(&right.0));

    let _ = writeln!(rendered, "\n## Wait Registrations");

    if let Some(competition) = instance.event_competition.as_ref() {
        let _ = writeln!(
            rendered,
            "Competition gateway: {}",
            render_bpmn_wait_node_id(process, competition.gateway_node_index)
        );
    }

    let wait_key = wait_lines
        .iter()
        .map(|(wait_id, _)| wait_id.as_str())
        .collect::<Vec<_>>()
        .join("|");
    let _ = writeln!(rendered, "Event fixture key: {wait_key}");

    for (_, line) in wait_lines {
        let _ = writeln!(rendered, "{line}");
    }
}

fn render_bpmn_wait_node_id(process: &BpmnProcessSpec, node_index: u32) -> String {
    bpmn_node_id_label(process, node_index)
}
