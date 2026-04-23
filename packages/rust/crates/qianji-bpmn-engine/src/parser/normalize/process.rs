use super::compensation::normalize_compensation_handlers;
use super::digest::process_digest_hex;
use super::event::normalize_events;
use super::node::normalize_nodes;
use crate::error::{BpmnEngineError, Result};
use crate::ir_edge_api::BpmnEdgeSpec;
use crate::ir_package_api::BpmnPackage;
use crate::ir_process_key::ProcessKey;
use crate::ir_process_spec::BpmnProcessSpec;
use crate::parser::import::{RawPackageDocument, RawProcess};
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
    let edges = normalize_edges(raw, &index_by_id);
    let nodes = normalize_nodes(raw, &index_by_id, &edges)?;
    let events = normalize_events(raw)?;
    let compensation_handlers = normalize_compensation_handlers(raw, &index_by_id)?;

    Ok(BpmnProcessSpec::new_with_compensation(
        ProcessKey::new(package_id, &raw.process_id, digest_hex),
        nodes,
        edges,
        events,
        compensation_handlers,
    ))
}

pub(super) fn normalize_node_index(index: usize, operation: &'static str) -> Result<u32> {
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

fn normalize_edges(raw: &RawProcess, index_by_id: &HashMap<String, u32>) -> Vec<BpmnEdgeSpec> {
    raw.flows
        .iter()
        .map(|flow| {
            let edge = BpmnEdgeSpec::new(
                index_by_id[&flow.source_ref],
                index_by_id[&flow.target_ref],
                flow.label.as_deref(),
            );
            match flow.condition_expression.as_deref() {
                Some(condition_expression) => edge.with_condition_expression(condition_expression),
                None => edge,
            }
        })
        .collect()
}
