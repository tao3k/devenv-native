//! BPMN normalization into immutable runtime-ready IR.

use super::import::{RawNode, RawPackageDocument, RawProcess, RawRepeatSpec};
use crate::error::{BpmnEngineError, Result};
use crate::ir::{
    BpmnEdgeSpec, BpmnEventSpec, BpmnNodeSpec, BpmnPackage, BpmnProcessSpec, BpmnRepeatSpec,
    BpmnSequentialMultiInstanceSpec, BpmnStandardLoopSpec, BpmnTimerKind, BpmnTimerSpec,
    ProcessKey,
};
use std::collections::HashMap;

pub(crate) fn normalize_package(raw: RawPackageDocument) -> Result<BpmnPackage> {
    let package_id = raw.package_id;
    let source_id = raw.source_id;
    let processes = raw
        .processes
        .iter()
        .map(|process| normalize_process(&package_id, &source_id, process))
        .collect::<Result<Vec<_>>>()?;
    Ok(BpmnPackage::new(package_id, processes))
}

fn normalize_process(
    package_id: &str,
    source_id: &str,
    raw: &RawProcess,
) -> Result<BpmnProcessSpec> {
    let digest_hex = process_digest_hex(package_id, source_id, raw);
    let index_by_id = build_node_index_by_id(raw)?;
    let nodes = normalize_nodes(raw, &index_by_id)?;
    let events = normalize_events(raw)?;
    let edges = normalize_edges(raw, &index_by_id);

    Ok(BpmnProcessSpec::new(
        ProcessKey::new(package_id, &raw.process_id, digest_hex),
        nodes,
        edges,
        events,
    ))
}

fn normalize_node_index(index: usize, operation: &'static str) -> Result<u32> {
    u32::try_from(index).map_err(|_| BpmnEngineError::UnsupportedOperation { operation })
}

fn build_node_index_by_id(raw: &RawProcess) -> Result<HashMap<String, u32>> {
    raw.nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            Ok((
                node.bpmn_id.clone(),
                normalize_node_index(index, "normalize_process_node_index_overflow")?,
            ))
        })
        .collect()
}

fn normalize_nodes(
    raw: &RawProcess,
    index_by_id: &HashMap<String, u32>,
) -> Result<Vec<BpmnNodeSpec>> {
    raw.nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let node_index =
                normalize_node_index(index, "normalize_process_node_spec_index_overflow")?;
            normalize_node(raw, node, node_index, index_by_id)
        })
        .collect()
}

fn normalize_node(
    raw: &RawProcess,
    node: &RawNode,
    node_index: u32,
    index_by_id: &HashMap<String, u32>,
) -> Result<BpmnNodeSpec> {
    let spec = BpmnNodeSpec::new(node_index, &node.bpmn_id, node.kind.clone());
    let spec = match node.gateway_kind.clone() {
        Some(gateway_kind) => spec.with_gateway_kind(gateway_kind),
        None => spec,
    };
    let spec = match node.decision.clone() {
        Some(decision) => spec.with_decision(decision),
        None => spec,
    };
    let spec = match &node.called_process_ref {
        Some(called_process_ref) => spec.with_called_process(called_process_ref),
        None => spec,
    };
    let spec = normalize_repeat_spec(raw, node, spec)?;
    attach_boundary_host(raw, node, spec, index_by_id)
}

fn normalize_repeat_spec(
    raw: &RawProcess,
    node: &RawNode,
    spec: BpmnNodeSpec,
) -> Result<BpmnNodeSpec> {
    match &node.repeat {
        Some(RawRepeatSpec::StandardLoop(loop_spec)) => {
            let repeat = BpmnStandardLoopSpec::new(loop_spec.test_before, loop_spec.loop_maximum);
            let repeat = match &loop_spec.loop_condition {
                Some(loop_condition) => repeat.with_loop_condition(loop_condition),
                None => repeat,
            };
            Ok(spec.with_repeat(BpmnRepeatSpec::StandardLoop(repeat)))
        }
        Some(RawRepeatSpec::SequentialMultiInstance(loop_spec)) => {
            Ok(spec.with_repeat(BpmnRepeatSpec::SequentialMultiInstance(
                BpmnSequentialMultiInstanceSpec::new(loop_spec.loop_cardinality.ok_or_else(
                    || BpmnEngineError::UnsupportedLoopConfiguration {
                        process_id: raw.process_id.clone(),
                        node_id: node.bpmn_id.clone(),
                        detail: "missing_loop_cardinality",
                    },
                )?),
            )))
        }
        None => Ok(spec),
    }
}

fn attach_boundary_host(
    raw: &RawProcess,
    node: &RawNode,
    spec: BpmnNodeSpec,
    index_by_id: &HashMap<String, u32>,
) -> Result<BpmnNodeSpec> {
    match &node.attached_to_ref {
        Some(attached_to_ref) => {
            let attached_to = index_by_id.get(attached_to_ref).copied().ok_or_else(|| {
                BpmnEngineError::UnknownBoundaryAttachment {
                    process_id: raw.process_id.clone(),
                    node_id: node.bpmn_id.clone(),
                    attached_to_node_id: attached_to_ref.clone(),
                }
            })?;
            Ok(spec.with_boundary_attachment(attached_to, node.cancel_activity))
        }
        None => Ok(spec),
    }
}

fn normalize_events(raw: &RawProcess) -> Result<Vec<BpmnEventSpec>> {
    let mut events = Vec::new();
    for (index, node) in raw.nodes.iter().enumerate() {
        let Some(event) = node.event.as_ref() else {
            continue;
        };
        let spec = BpmnEventSpec::new(
            normalize_node_index(index, "normalize_process_event_index_overflow")?,
            event.kind.clone(),
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

fn normalize_edges(raw: &RawProcess, index_by_id: &HashMap<String, u32>) -> Vec<BpmnEdgeSpec> {
    raw.flows
        .iter()
        .map(|flow| {
            BpmnEdgeSpec::new(
                index_by_id[&flow.source_ref],
                index_by_id[&flow.target_ref],
                flow.label.as_deref(),
            )
        })
        .collect()
}

fn process_digest_hex(package_id: &str, source_id: &str, raw: &RawProcess) -> String {
    let mut material = String::new();
    material.push_str(package_id);
    material.push('\n');
    material.push_str(source_id);
    material.push('\n');
    material.push_str(&raw.process_id);
    material.push('\n');
    for node in &raw.nodes {
        append_node_digest(&mut material, node);
    }
    for flow in &raw.flows {
        append_flow_digest(&mut material, flow);
    }
    format!("{:x}", md5::compute(material))
}

fn append_node_digest(material: &mut String, node: &RawNode) {
    material.push_str(&node.bpmn_id);
    material.push(':');
    material.push_str(node_kind_name(&node.kind));
    if let Some(gateway_kind) = &node.gateway_kind {
        material.push(':');
        material.push_str(gateway_kind_name(gateway_kind));
    }
    if let Some(decision) = &node.decision {
        material.push(':');
        material.push_str(decision.decision_id.as_ref());
    }
    if let Some(called_process_ref) = &node.called_process_ref {
        material.push(':');
        material.push_str("called_process=");
        material.push_str(called_process_ref);
    }
    if let Some(repeat) = &node.repeat {
        append_repeat_digest(material, repeat);
    }
    if let Some(attached_to_ref) = &node.attached_to_ref {
        material.push(':');
        material.push_str("attached_to=");
        material.push_str(attached_to_ref);
        material.push(':');
        material.push_str(if node.cancel_activity {
            "interrupting"
        } else {
            "non_interrupting"
        });
    }
    if let Some(event) = &node.event {
        material.push(':');
        material.push_str(event_kind_name(&event.kind));
        if let Some(reference_id) = &event.reference_id {
            material.push(':');
            material.push_str(reference_id);
        }
        if let Some(name) = &event.name {
            material.push(':');
            material.push_str(name);
        }
        if let Some(timer) = &event.timer {
            material.push(':');
            material.push_str(timer_kind_name(&timer.kind));
            material.push(':');
            material.push_str(&timer.expression);
        }
    }
    material.push('\n');
}

fn append_repeat_digest(material: &mut String, repeat: &RawRepeatSpec) {
    match repeat {
        RawRepeatSpec::StandardLoop(loop_spec) => {
            material.push(':');
            material.push_str("repeat=standard_loop");
            material.push(':');
            material.push_str(if loop_spec.test_before {
                "test_before"
            } else {
                "test_after"
            });
            if let Some(loop_maximum) = loop_spec.loop_maximum {
                material.push(':');
                material.push_str("loop_maximum=");
                material.push_str(&loop_maximum.to_string());
            }
            if let Some(loop_condition) = &loop_spec.loop_condition {
                material.push(':');
                material.push_str("loop_condition=");
                material.push_str(loop_condition);
            }
        }
        RawRepeatSpec::SequentialMultiInstance(loop_spec) => {
            material.push(':');
            material.push_str("repeat=sequential_multi_instance");
            if let Some(loop_cardinality) = loop_spec.loop_cardinality {
                material.push(':');
                material.push_str("loop_cardinality=");
                material.push_str(&loop_cardinality.to_string());
            }
        }
    }
}

fn append_flow_digest(material: &mut String, flow: &super::import::RawSequenceFlow) {
    material.push_str(&flow.flow_id);
    material.push(':');
    material.push_str(&flow.source_ref);
    material.push(':');
    material.push_str(&flow.target_ref);
    if let Some(label) = &flow.label {
        material.push(':');
        material.push_str(label);
    }
    material.push('\n');
}

fn node_kind_name(kind: &crate::ir::BpmnNodeKind) -> &'static str {
    match kind {
        crate::ir::BpmnNodeKind::StartEvent => "start_event",
        crate::ir::BpmnNodeKind::EndEvent => "end_event",
        crate::ir::BpmnNodeKind::IntermediateCatchEvent => "intermediate_catch_event",
        crate::ir::BpmnNodeKind::BoundaryEvent => "boundary_event",
        crate::ir::BpmnNodeKind::ServiceTask => "service_task",
        crate::ir::BpmnNodeKind::UserTask => "user_task",
        crate::ir::BpmnNodeKind::ManualTask => "manual_task",
        crate::ir::BpmnNodeKind::BusinessRuleTask => "business_rule_task",
        crate::ir::BpmnNodeKind::Gateway => "gateway",
        crate::ir::BpmnNodeKind::SubProcess => "sub_process",
    }
}

fn event_kind_name(kind: &crate::ir::BpmnEventKind) -> &'static str {
    match kind {
        crate::ir::BpmnEventKind::Timer => "timer",
        crate::ir::BpmnEventKind::Message => "message",
        crate::ir::BpmnEventKind::Signal => "signal",
        crate::ir::BpmnEventKind::Conditional => "conditional",
    }
}

trait RawNodeEventLabelFallback {
    fn event_label_fallback(&self) -> &str;
}

impl RawNodeEventLabelFallback for super::import::RawNode {
    fn event_label_fallback(&self) -> &str {
        &self.bpmn_id
    }
}

fn gateway_kind_name(kind: &crate::ir::BpmnGatewayKind) -> &'static str {
    match kind {
        crate::ir::BpmnGatewayKind::Parallel => "parallel",
        crate::ir::BpmnGatewayKind::Exclusive => "exclusive",
        crate::ir::BpmnGatewayKind::EventBased => "event_based",
    }
}

fn timer_kind_name(kind: &BpmnTimerKind) -> &'static str {
    match kind {
        BpmnTimerKind::Date => "date",
        BpmnTimerKind::Duration => "duration",
        BpmnTimerKind::Cycle => "cycle",
    }
}
