use super::process::normalize_node_index;
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
        let spec = BpmnEventSpec::new(
            normalize_node_index(index, "normalize_process_event_index_overflow")?,
            event.kind,
        );
        let spec = match &event.reference_id {
            Some(reference_id) => spec.with_reference_id(reference_id),
            None => spec,
        };
        let spec = match &event.timer {
            Some(timer) => {
                spec.with_timer(BpmnTimerSpec::new(timer.kind.clone(), &timer.expression))
            }
            None => spec,
        };
        let name = event.name.as_deref().unwrap_or(node.event_label_fallback());
        events.push(spec.with_name(name));
    }
    Ok(events)
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
            name: None,
            timer: None,
        })
}
