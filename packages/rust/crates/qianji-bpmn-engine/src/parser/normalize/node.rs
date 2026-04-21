use super::process::normalize_node_index;
use super::repeat::normalize_repeat_spec;
use crate::error::{BpmnEngineError, Result};
use crate::ir_edge_api::BpmnEdgeSpec;
use crate::ir_node_api::{BpmnNodeSpec, BpmnSubProcessKind};
use crate::parser::import::{RawNode, RawProcess, RawSubProcessKind};
use crate::parser::validate::resolve_structured_inclusive_join;
use std::collections::HashMap;

pub(super) fn normalize_nodes(
    raw: &RawProcess,
    index_by_id: &HashMap<String, u32>,
    edges: &[BpmnEdgeSpec],
) -> Result<Vec<BpmnNodeSpec>> {
    raw.nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let node_index =
                normalize_node_index(index, "normalize_process_node_spec_index_overflow")?;
            normalize_node(raw, node, node_index, index_by_id, edges)
        })
        .collect()
}

fn normalize_node(
    raw: &RawProcess,
    node: &RawNode,
    node_index: u32,
    index_by_id: &HashMap<String, u32>,
    edges: &[BpmnEdgeSpec],
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
    let spec = match node.subprocess_kind {
        Some(kind) => spec.with_subprocess_kind(normalize_subprocess_kind(kind)),
        None => spec,
    };
    let spec = normalize_repeat_spec(raw, node, spec)?;
    let spec = if node.is_for_compensation {
        spec.with_compensation_marker(true)
    } else {
        spec
    };
    let spec = attach_default_outgoing_edge(raw, node, spec, index_by_id, edges)?;
    let spec = attach_inclusive_join_node(raw, node, spec, index_by_id)?;
    attach_boundary_host(raw, node, spec, index_by_id)
}

fn attach_default_outgoing_edge(
    raw: &RawProcess,
    node: &RawNode,
    spec: BpmnNodeSpec,
    index_by_id: &HashMap<String, u32>,
    edges: &[BpmnEdgeSpec],
) -> Result<BpmnNodeSpec> {
    let Some(default_flow_ref) = node.default_flow_ref.as_deref() else {
        return Ok(spec);
    };
    let Some(source_index) = index_by_id.get(node.bpmn_id.as_str()).copied() else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "normalize_node_missing_source_index_for_default_flow",
        });
    };
    let edge_index = raw
        .flows
        .iter()
        .position(|flow| flow.flow_id == default_flow_ref)
        .ok_or_else(|| BpmnEngineError::UnknownSequenceFlowEndpoint {
            process_id: raw.process_id.clone(),
            flow_id: default_flow_ref.to_string(),
            endpoint: "default",
            node_id: node.bpmn_id.clone(),
        })?;
    let normalized_edge_index =
        normalize_node_index(edge_index, "normalize_default_flow_edge_index_overflow")?;
    if edges[edge_index].from != source_index {
        return Err(BpmnEngineError::UnknownSequenceFlowEndpoint {
            process_id: raw.process_id.clone(),
            flow_id: default_flow_ref.to_string(),
            endpoint: "default",
            node_id: node.bpmn_id.clone(),
        });
    }
    Ok(spec.with_default_outgoing_edge(normalized_edge_index))
}

fn attach_inclusive_join_node(
    raw: &RawProcess,
    node: &RawNode,
    spec: BpmnNodeSpec,
    index_by_id: &HashMap<String, u32>,
) -> Result<BpmnNodeSpec> {
    let Some(join_node_id) = resolve_structured_inclusive_join(raw, node)? else {
        return Ok(spec);
    };
    let join_node_index = index_by_id.get(join_node_id.as_str()).copied().ok_or(
        BpmnEngineError::UnsupportedOperation {
            operation: "normalize_inclusive_gateway_missing_join_index",
        },
    )?;
    Ok(spec.with_inclusive_join_node(join_node_index))
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

fn normalize_subprocess_kind(kind: RawSubProcessKind) -> BpmnSubProcessKind {
    match kind {
        RawSubProcessKind::CallActivity => BpmnSubProcessKind::CallActivity,
        RawSubProcessKind::EmbeddedSubProcess => BpmnSubProcessKind::Embedded,
        RawSubProcessKind::Transaction => BpmnSubProcessKind::Transaction,
    }
}
