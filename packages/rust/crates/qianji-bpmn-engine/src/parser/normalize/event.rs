use super::normalize_node_index;
use crate::error::Result;
use crate::ir_event_api::{BpmnEventSpec, BpmnTimerSpec};
use crate::parser::import::{RawEventSpec, RawNode, RawProcess};

pub(super) fn normalize_events(raw: &RawProcess) -> Result<Vec<BpmnEventSpec>> {
    let mut events = Vec::new();
    for (index, node) in raw.nodes.iter().enumerate() {
        let event = node
            .event
            .clone()
            .or_else(|| fallback_task_message_event(node));
        let Some(event) = event else {
            continue;
        };
        events.push(normalize_event_spec(
            normalize_node_index(index, "normalize_process_event_index_overflow")?,
            node.event_label_fallback(),
            &event,
        ));
    }
    Ok(events)
}

pub(super) fn normalize_event_spec(
    node_index: crate::ir_index_api::BpmnNodeIndex,
    label_fallback: &str,
    event: &RawEventSpec,
) -> BpmnEventSpec {
    let spec = BpmnEventSpec::new(node_index, event.kind.clone());
    let spec = match &event.reference_id {
        Some(reference_id) => spec.with_reference_id(reference_id),
        None => spec,
    };
    let spec = spec.with_wait_for_completion(event.wait_for_completion);
    let spec = match &event.timer {
        Some(timer) => spec.with_timer(BpmnTimerSpec::new(timer.kind.clone(), &timer.expression)),
        None => spec,
    };
    let spec = match &event.condition_expression {
        Some(condition_expression) => spec.with_condition_expression(condition_expression),
        None => spec,
    };
    let name = event.name.as_deref().unwrap_or(label_fallback);
    spec.with_name(name)
}

trait RawNodeEventLabelFallback {
    fn event_label_fallback(&self) -> &str;
}

impl RawNodeEventLabelFallback for RawNode {
    fn event_label_fallback(&self) -> &str {
        &self.bpmn_id
    }
}

fn fallback_task_message_event(node: &RawNode) -> Option<RawEventSpec> {
    if !matches!(
        node.kind,
        crate::ir_node_api::BpmnNodeKind::SendTask | crate::ir_node_api::BpmnNodeKind::ReceiveTask
    ) {
        return None;
    }
    node.task_message_ref
        .as_ref()
        .map(|reference_id| RawEventSpec {
            kind: crate::ir_event_api::BpmnEventKind::Message,
            reference_id: Some(reference_id.clone()),
            wait_for_completion: true,
            name: None,
            timer: None,
            condition_expression: None,
        })
}
