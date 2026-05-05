use super::compensation::normalize_compensation_handlers;
use super::digest::process_digest_hex;
use super::event::{normalize_event_spec, normalize_events};
use super::node::normalize_nodes;
use super::normalize_node_index;
use crate::error::{BpmnEngineError, Result};
use crate::ir_data_api::BpmnDataObjectBindingSpec;
use crate::ir_edge_api::BpmnEdgeSpec;
use crate::ir_package_api::BpmnPackage;
use crate::ir_process_key::ProcessKey;
use crate::ir_process_spec::BpmnProcessSpec;
use crate::parser::import::{RawPackageDocument, RawProcess, RawSubProcessKind};
use std::collections::{HashMap, HashSet};

pub(crate) fn normalize_package(raw: RawPackageDocument) -> Result<BpmnPackage> {
    let package_id = raw.package_id;
    let source_id = raw.source_id;
    let process_by_id = raw
        .processes
        .iter()
        .map(|process| (process.process_id.as_str(), process))
        .collect::<HashMap<_, _>>();
    let processes = raw
        .processes
        .iter()
        .map(|process| normalize_process(&package_id, &source_id, process, &process_by_id))
        .collect::<Result<Vec<_>>>()?;
    Ok(BpmnPackage::new(package_id, processes))
}

fn normalize_process(
    package_id: &str,
    source_id: &str,
    raw: &RawProcess,
    process_by_id: &HashMap<&str, &RawProcess>,
) -> Result<BpmnProcessSpec> {
    let digest_hex = process_digest_hex(package_id, source_id, raw);
    let index_by_id = build_node_index_by_id(raw)?;
    let edges = normalize_edges(raw, &index_by_id);
    let nodes = normalize_nodes(raw, &index_by_id, &edges)?;
    let mut events = normalize_events(raw)?;
    events.extend(normalize_event_subprocess_owner_events(
        raw,
        &index_by_id,
        process_by_id,
    )?);
    let compensation_handlers = normalize_compensation_handlers(raw, &index_by_id)?;
    let data_object_bindings = normalize_data_object_bindings(raw)?;

    Ok(BpmnProcessSpec::new_with_compensation(
        ProcessKey::new(package_id, &raw.process_id, digest_hex),
        nodes,
        edges,
        events,
        compensation_handlers,
    ))
    .map(|process| process.with_data_object_bindings(data_object_bindings))
}

fn normalize_event_subprocess_owner_events(
    raw: &RawProcess,
    index_by_id: &HashMap<String, u32>,
    process_by_id: &HashMap<&str, &RawProcess>,
) -> Result<Vec<crate::ir_event_api::BpmnEventSpec>> {
    let mut events = Vec::new();
    for owner in raw
        .nodes
        .iter()
        .filter(|node| node.subprocess_kind == Some(RawSubProcessKind::EventSubProcess))
    {
        let owner_index = index_by_id.get(&owner.bpmn_id).copied().ok_or(
            BpmnEngineError::UnsupportedOperation {
                operation: "normalize_event_subprocess_missing_owner_index",
            },
        )?;
        let called_process_id =
            owner
                .called_process_ref
                .as_ref()
                .ok_or(BpmnEngineError::UnsupportedOperation {
                    operation: "normalize_event_subprocess_missing_child_process",
                })?;
        let child = process_by_id.get(called_process_id.as_str()).ok_or(
            BpmnEngineError::UnsupportedOperation {
                operation: "normalize_event_subprocess_unknown_child_process",
            },
        )?;
        let start = child
            .nodes
            .iter()
            .find(|node| node.kind == crate::ir_node_api::BpmnNodeKind::StartEvent)
            .ok_or(BpmnEngineError::UnsupportedOperation {
                operation: "normalize_event_subprocess_missing_start_event",
            })?;
        let event = start
            .event
            .as_ref()
            .ok_or(BpmnEngineError::UnsupportedOperation {
                operation: "normalize_event_subprocess_missing_start_event_definition",
            })?;
        events.push(normalize_event_spec(owner_index, &start.bpmn_id, event));
    }
    Ok(events)
}

fn normalize_data_object_bindings(raw: &RawProcess) -> Result<Vec<BpmnDataObjectBindingSpec>> {
    let object_ids = raw
        .data_objects
        .iter()
        .map(|data_object| data_object.id.as_str())
        .collect::<HashSet<_>>();
    let mut bindings = raw
        .data_objects
        .iter()
        .map(|data_object| BpmnDataObjectBindingSpec::object(&data_object.id))
        .collect::<Vec<_>>();

    for reference in &raw.data_object_references {
        if !object_ids.contains(reference.data_object_ref.as_str()) {
            return Err(BpmnEngineError::UnknownDataObjectReference {
                process_id: raw.process_id.clone(),
                reference_id: reference.id.clone(),
                data_object_ref: reference.data_object_ref.clone(),
            });
        }
        bindings.push(BpmnDataObjectBindingSpec::reference(
            &reference.id,
            &reference.data_object_ref,
        ));
    }

    Ok(bindings)
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
