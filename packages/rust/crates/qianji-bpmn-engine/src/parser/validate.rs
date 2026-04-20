//! BPMN validation for the bounded supported subset.

use super::import::RawPackageDocument;
use super::import::RawRepeatSpec;
use crate::error::{BpmnEngineError, Result};
use crate::ir::{BpmnEventKind, BpmnGatewayKind, BpmnNodeKind};
use std::collections::{HashMap, HashSet};

pub(crate) fn validate_raw_package(raw: &RawPackageDocument) -> Result<()> {
    ensure_process_definitions(raw)?;
    let all_process_ids = raw
        .processes
        .iter()
        .map(|process| process.process_id.as_str())
        .collect::<HashSet<_>>();
    let mut seen_process_ids = HashSet::new();
    for process in &raw.processes {
        ensure_unique_process_id(raw, process, &mut seen_process_ids)?;
        let node_ids = collect_node_ids(process)?;
        validate_process_topology(process, &all_process_ids, &node_ids)?;
        validate_sequence_flows(process, &node_ids)?;
        validate_standard_loops(process)?;
        validate_sequential_multi_instances(process)?;
        validate_event_based_gateways(process, &node_ids)?;
    }

    detect_recursive_call_activity(raw)?;

    Ok(())
}

fn ensure_process_definitions(raw: &RawPackageDocument) -> Result<()> {
    if raw.processes.is_empty() {
        return Err(BpmnEngineError::MissingProcessDefinitions {
            source_id: raw.source_id.clone(),
        });
    }
    Ok(())
}

fn ensure_unique_process_id<'a>(
    raw: &RawPackageDocument,
    process: &'a super::import::RawProcess,
    seen_process_ids: &mut HashSet<&'a str>,
) -> Result<()> {
    if seen_process_ids.insert(process.process_id.as_str()) {
        return Ok(());
    }
    Err(BpmnEngineError::DuplicateProcessId {
        package_id: raw.package_id.clone(),
        process_id: process.process_id.clone(),
    })
}

fn collect_node_ids(process: &super::import::RawProcess) -> Result<HashSet<&str>> {
    let mut seen_node_ids = HashSet::new();
    let mut node_ids = HashSet::new();
    let mut has_start_event = false;
    let mut has_end_event = false;

    for node in &process.nodes {
        if !seen_node_ids.insert(node.bpmn_id.as_str()) {
            return Err(BpmnEngineError::DuplicateNodeId {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
            });
        }
        node_ids.insert(node.bpmn_id.as_str());
        has_start_event |= matches!(node.kind, BpmnNodeKind::StartEvent);
        has_end_event |= matches!(node.kind, BpmnNodeKind::EndEvent);
    }

    if !has_start_event {
        return Err(BpmnEngineError::MissingRequiredProcessElement {
            process_id: process.process_id.clone(),
            element: "start_event",
        });
    }
    if !has_end_event {
        return Err(BpmnEngineError::MissingRequiredProcessElement {
            process_id: process.process_id.clone(),
            element: "end_event",
        });
    }

    Ok(node_ids)
}

fn validate_process_topology(
    process: &super::import::RawProcess,
    all_process_ids: &HashSet<&str>,
    node_ids: &HashSet<&str>,
) -> Result<()> {
    let mut boundary_attachments = HashSet::new();
    for node in &process.nodes {
        validate_node_event_shape(process, node)?;
        if node.kind == BpmnNodeKind::SubProcess {
            validate_called_process_reference(process, node, all_process_ids)?;
        }
        if node.kind == BpmnNodeKind::BoundaryEvent {
            validate_boundary_event(process, node, node_ids, &mut boundary_attachments)?;
        }
    }
    Ok(())
}

fn validate_node_event_shape(
    process: &super::import::RawProcess,
    node: &super::import::RawNode,
) -> Result<()> {
    if matches!(
        node.kind,
        BpmnNodeKind::IntermediateCatchEvent | BpmnNodeKind::BoundaryEvent
    ) && node.event.is_none()
    {
        return Err(BpmnEngineError::MissingRequiredNodeElement {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            element: "event_definition",
        });
    }

    if let Some(event) = &node.event
        && event.kind == BpmnEventKind::Timer
        && event
            .timer
            .as_ref()
            .is_none_or(|timer| timer.expression.trim().is_empty())
    {
        return Err(BpmnEngineError::MissingRequiredNodeElement {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            element: "timer_expression",
        });
    }

    Ok(())
}

fn validate_called_process_reference(
    process: &super::import::RawProcess,
    node: &super::import::RawNode,
    all_process_ids: &HashSet<&str>,
) -> Result<()> {
    let called_process_id = node.called_process_ref.as_deref().ok_or_else(|| {
        BpmnEngineError::MissingRequiredNodeElement {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            element: "called_process",
        }
    })?;
    if all_process_ids.contains(called_process_id) {
        return Ok(());
    }
    Err(BpmnEngineError::UnknownCalledProcess {
        process_id: process.process_id.clone(),
        node_id: node.bpmn_id.clone(),
        called_process_id: called_process_id.to_string(),
    })
}

fn validate_boundary_event(
    process: &super::import::RawProcess,
    node: &super::import::RawNode,
    node_ids: &HashSet<&str>,
    boundary_attachments: &mut HashSet<String>,
) -> Result<()> {
    let attached_to_ref = node.attached_to_ref.as_deref().ok_or_else(|| {
        BpmnEngineError::MissingRequiredNodeElement {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            element: "attached_to_ref",
        }
    })?;

    if !node_ids.contains(attached_to_ref) {
        return Err(BpmnEngineError::UnknownBoundaryAttachment {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            attached_to_node_id: attached_to_ref.to_string(),
        });
    }
    if !boundary_attachments.insert(attached_to_ref.to_string()) {
        return Err(BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "multiple_boundary_events_for_attached_node",
        });
    }
    if !node.cancel_activity {
        return Err(BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "non_interrupting_boundary_event",
        });
    }

    let attached_node = process
        .nodes
        .iter()
        .find(|candidate| candidate.bpmn_id == attached_to_ref)
        .ok_or_else(|| BpmnEngineError::UnknownBoundaryAttachment {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            attached_to_node_id: attached_to_ref.to_string(),
        })?;
    if !matches!(
        attached_node.kind,
        BpmnNodeKind::ServiceTask
            | BpmnNodeKind::UserTask
            | BpmnNodeKind::ManualTask
            | BpmnNodeKind::BusinessRuleTask
    ) {
        return Err(BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "unsupported_boundary_attachment_kind",
        });
    }
    if node.event.as_ref().map(|event| &event.kind) != Some(&BpmnEventKind::Timer) {
        return Err(BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "unsupported_boundary_event_kind",
        });
    }

    Ok(())
}

fn validate_sequence_flows(
    process: &super::import::RawProcess,
    node_ids: &HashSet<&str>,
) -> Result<()> {
    let mut seen_flow_ids = HashSet::new();
    for flow in &process.flows {
        if !seen_flow_ids.insert(flow.flow_id.as_str()) {
            return Err(BpmnEngineError::DuplicateSequenceFlowId {
                process_id: process.process_id.clone(),
                flow_id: flow.flow_id.clone(),
            });
        }
        if !node_ids.contains(flow.source_ref.as_str()) {
            return Err(BpmnEngineError::UnknownSequenceFlowEndpoint {
                process_id: process.process_id.clone(),
                flow_id: flow.flow_id.clone(),
                endpoint: "source",
                node_id: flow.source_ref.clone(),
            });
        }
        if !node_ids.contains(flow.target_ref.as_str()) {
            return Err(BpmnEngineError::UnknownSequenceFlowEndpoint {
                process_id: process.process_id.clone(),
                flow_id: flow.flow_id.clone(),
                endpoint: "target",
                node_id: flow.target_ref.clone(),
            });
        }
    }
    Ok(())
}

fn validate_standard_loops(process: &super::import::RawProcess) -> Result<()> {
    for node in &process.nodes {
        let Some(RawRepeatSpec::StandardLoop(loop_spec)) = &node.repeat else {
            continue;
        };

        if !matches!(
            node.kind,
            BpmnNodeKind::ServiceTask
                | BpmnNodeKind::UserTask
                | BpmnNodeKind::ManualTask
                | BpmnNodeKind::BusinessRuleTask
        ) {
            return Err(BpmnEngineError::UnsupportedLoopConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "unsupported_standard_loop_host_kind",
            });
        }

        let loop_condition = loop_spec
            .loop_condition
            .as_deref()
            .map(str::trim)
            .filter(|condition| !condition.is_empty());
        if loop_spec.loop_maximum.is_none() && loop_condition.is_none() {
            return Err(BpmnEngineError::UnsupportedLoopConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "missing_loop_maximum_or_condition",
            });
        }

        if loop_spec.loop_maximum == Some(0) {
            return Err(BpmnEngineError::UnsupportedLoopConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "non_positive_loop_maximum",
            });
        }

        if let Some(loop_condition) = loop_condition
            && !is_supported_standard_loop_condition(loop_condition)
        {
            return Err(BpmnEngineError::UnsupportedLoopConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "unsupported_loop_condition_expression",
            });
        }
    }

    Ok(())
}

fn validate_sequential_multi_instances(process: &super::import::RawProcess) -> Result<()> {
    for node in &process.nodes {
        let Some(RawRepeatSpec::SequentialMultiInstance(multi_instance_spec)) = &node.repeat else {
            continue;
        };

        if !matches!(
            node.kind,
            BpmnNodeKind::ServiceTask
                | BpmnNodeKind::UserTask
                | BpmnNodeKind::ManualTask
                | BpmnNodeKind::BusinessRuleTask
        ) {
            return Err(BpmnEngineError::UnsupportedLoopConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "unsupported_multi_instance_host_kind",
            });
        }

        if multi_instance_spec.loop_cardinality.is_none() {
            return Err(BpmnEngineError::UnsupportedLoopConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "missing_loop_cardinality",
            });
        }
    }

    Ok(())
}

fn validate_event_based_gateways(
    process: &super::import::RawProcess,
    node_ids: &HashSet<&str>,
) -> Result<()> {
    let node_by_id = process
        .nodes
        .iter()
        .map(|node| (node.bpmn_id.as_str(), node))
        .collect::<HashMap<_, _>>();

    for gateway in process.nodes.iter().filter(|node| {
        node.kind == BpmnNodeKind::Gateway && node.gateway_kind == Some(BpmnGatewayKind::EventBased)
    }) {
        let outgoing_targets = process
            .flows
            .iter()
            .filter(|flow| flow.source_ref == gateway.bpmn_id)
            .map(|flow| flow.target_ref.as_str())
            .collect::<Vec<_>>();

        if outgoing_targets.len() < 2 {
            return Err(BpmnEngineError::UnsupportedEventBasedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: gateway.bpmn_id.clone(),
                detail: "insufficient_outgoing_waits",
            });
        }

        for target_id in outgoing_targets {
            if !node_ids.contains(target_id) {
                continue;
            }

            let Some(target) = node_by_id.get(target_id) else {
                return Err(BpmnEngineError::UnknownSequenceFlowEndpoint {
                    process_id: process.process_id.clone(),
                    flow_id: gateway.bpmn_id.clone(),
                    endpoint: "target",
                    node_id: target_id.to_string(),
                });
            };
            if target.kind != BpmnNodeKind::IntermediateCatchEvent {
                return Err(BpmnEngineError::UnsupportedEventBasedGatewayConfiguration {
                    process_id: process.process_id.clone(),
                    node_id: gateway.bpmn_id.clone(),
                    detail: "unsupported_wait_target_kind",
                });
            }

            let Some(event) = target.event.as_ref() else {
                return Err(BpmnEngineError::MissingRequiredNodeElement {
                    process_id: process.process_id.clone(),
                    node_id: target.bpmn_id.clone(),
                    element: "event_definition",
                });
            };
            if !matches!(
                event.kind,
                BpmnEventKind::Message | BpmnEventKind::Signal | BpmnEventKind::Timer
            ) {
                return Err(BpmnEngineError::UnsupportedEventBasedGatewayConfiguration {
                    process_id: process.process_id.clone(),
                    node_id: gateway.bpmn_id.clone(),
                    detail: "unsupported_wait_event_kind",
                });
            }
        }
    }

    Ok(())
}

fn detect_recursive_call_activity(raw: &RawPackageDocument) -> Result<()> {
    let call_graph = raw
        .processes
        .iter()
        .map(|process| {
            let edges = process
                .nodes
                .iter()
                .filter(|node| node.kind == BpmnNodeKind::SubProcess)
                .filter_map(|node| {
                    node.called_process_ref
                        .as_ref()
                        .map(|called_process_id| (called_process_id.clone(), node.bpmn_id.clone()))
                })
                .collect::<Vec<_>>();
            (process.process_id.clone(), edges)
        })
        .collect::<std::collections::HashMap<_, _>>();

    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    for process_id in call_graph.keys() {
        if let Some(error) = detect_recursive_call_activity_from(
            process_id,
            &call_graph,
            &mut visiting,
            &mut visited,
        ) {
            return Err(error);
        }
    }
    Ok(())
}

fn detect_recursive_call_activity_from(
    process_id: &str,
    call_graph: &std::collections::HashMap<String, Vec<(String, String)>>,
    visiting: &mut HashSet<String>,
    visited: &mut HashSet<String>,
) -> Option<BpmnEngineError> {
    if visited.contains(process_id) {
        return None;
    }

    visiting.insert(process_id.to_string());
    if let Some(edges) = call_graph.get(process_id) {
        for (called_process_id, node_id) in edges {
            if visiting.contains(called_process_id) {
                return Some(BpmnEngineError::UnsupportedSubProcessConfiguration {
                    process_id: process_id.to_string(),
                    node_id: node_id.clone(),
                    detail: "recursive_call_activity",
                });
            }
            if let Some(error) = detect_recursive_call_activity_from(
                called_process_id,
                call_graph,
                visiting,
                visited,
            ) {
                return Some(error);
            }
        }
    }
    visiting.remove(process_id);
    visited.insert(process_id.to_string());
    None
}

fn is_supported_standard_loop_condition(condition: &str) -> bool {
    let trimmed = condition.trim();
    let path = trimmed.strip_prefix("not ").map_or(trimmed, str::trim);
    !path.is_empty() && path.split('.').all(is_identifier_segment)
}

fn is_identifier_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}
