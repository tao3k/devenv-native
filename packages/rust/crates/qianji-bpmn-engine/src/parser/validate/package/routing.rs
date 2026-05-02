use crate::error::{BpmnEngineError, Result};
use crate::ir_event_api::BpmnEventKind;
use crate::ir_node_api::{BpmnGatewayKind, BpmnNodeKind};
use crate::parser::import::{RawNode, RawProcess, RawRepeatSpec, RawSequenceFlow};
use crate::repeat_condition::{
    is_supported_gateway_condition, is_supported_multi_instance_completion_condition,
};
use std::collections::{HashMap, HashSet};

pub(super) fn validate_sequence_flows(
    process: &RawProcess,
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

pub(super) fn validate_task_routing(process: &RawProcess) -> Result<()> {
    let mut outgoing_counts = HashMap::<&str, usize>::new();
    for flow in &process.flows {
        *outgoing_counts.entry(flow.source_ref.as_str()).or_default() += 1;
    }

    for node in &process.nodes {
        if node.is_for_compensation || !requires_single_outgoing_task_route(&node.kind) {
            continue;
        }
        let outgoing_count = outgoing_counts
            .get(node.bpmn_id.as_str())
            .copied()
            .unwrap_or_default();
        if outgoing_count != 1 {
            return Err(BpmnEngineError::UnsupportedTaskConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "task_requires_single_outgoing",
            });
        }
    }
    Ok(())
}

fn requires_single_outgoing_task_route(kind: &BpmnNodeKind) -> bool {
    matches!(
        kind,
        BpmnNodeKind::ServiceTask
            | BpmnNodeKind::ScriptTask
            | BpmnNodeKind::UserTask
            | BpmnNodeKind::ManualTask
            | BpmnNodeKind::BusinessRuleTask
            | BpmnNodeKind::SendTask
            | BpmnNodeKind::ReceiveTask
    )
}

pub(super) fn validate_gateways(process: &RawProcess) -> Result<()> {
    let flow_by_id = process
        .flows
        .iter()
        .map(|flow| (flow.flow_id.as_str(), flow))
        .collect::<HashMap<_, _>>();

    for node in &process.nodes {
        let outgoing = process
            .flows
            .iter()
            .filter(|flow| flow.source_ref == node.bpmn_id)
            .collect::<Vec<_>>();

        for flow in &outgoing {
            if flow.condition_expression.is_none() {
                continue;
            }
            if !matches!(
                node.gateway_kind,
                Some(BpmnGatewayKind::Exclusive | BpmnGatewayKind::Inclusive)
            ) {
                return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                    process_id: process.process_id.clone(),
                    node_id: node.bpmn_id.clone(),
                    detail: "condition_expression_requires_conditional_gateway",
                });
            }
        }

        if !matches!(
            node.gateway_kind,
            Some(BpmnGatewayKind::Exclusive | BpmnGatewayKind::Inclusive)
        ) {
            continue;
        }

        validate_conditional_gateway_defaults_and_conditions(
            process,
            node,
            &outgoing,
            &flow_by_id,
        )?;

        if node.gateway_kind == Some(BpmnGatewayKind::Inclusive) {
            validate_structured_inclusive_gateway(process, node, &outgoing)?;
        }
    }

    Ok(())
}

fn validate_conditional_gateway_defaults_and_conditions<'a>(
    process: &RawProcess,
    node: &RawNode,
    outgoing: &[&'a RawSequenceFlow],
    flow_by_id: &HashMap<&'a str, &'a RawSequenceFlow>,
) -> Result<()> {
    if outgoing.len() <= 1 {
        if node.default_flow_ref.is_some() {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "default_flow_requires_multiple_outgoing",
            });
        }
        return Ok(());
    }

    let default_flow_id = node.default_flow_ref.as_deref();
    if let Some(default_flow_id) = default_flow_id {
        let Some(default_flow) = flow_by_id.get(default_flow_id).copied() else {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "unknown_default_flow",
            });
        };
        if default_flow.source_ref != node.bpmn_id {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "default_flow_not_outgoing",
            });
        }
        if default_flow.condition_expression.is_some() {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "default_flow_must_not_have_condition_expression",
            });
        }
    }

    for flow in outgoing {
        if Some(flow.flow_id.as_str()) == default_flow_id {
            continue;
        }
        let Some(condition_expression) = flow.condition_expression.as_deref() else {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "missing_condition_expression",
            });
        };
        if !is_supported_gateway_condition(condition_expression) {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "unsupported_condition_expression",
            });
        }
    }

    Ok(())
}

fn validate_structured_inclusive_gateway(
    process: &RawProcess,
    node: &RawNode,
    outgoing: &[&RawSequenceFlow],
) -> Result<()> {
    let incoming_len = process
        .flows
        .iter()
        .filter(|flow| flow.target_ref == node.bpmn_id)
        .count();

    if incoming_len == 1 && outgoing.len() > 1 {
        let _ = resolve_structured_inclusive_join(process, node)?;
        return Ok(());
    }

    if incoming_len > 1 && outgoing.len() == 1 {
        if node.default_flow_ref.is_some() {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "inclusive_join_default_not_supported",
            });
        }
        if outgoing[0].condition_expression.is_some() {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "inclusive_join_condition_expression_not_supported",
            });
        }
        return Ok(());
    }

    Err(BpmnEngineError::UnsupportedGatewayConfiguration {
        process_id: process.process_id.clone(),
        node_id: node.bpmn_id.clone(),
        detail: "inclusive_gateway_requires_structured_split_or_join",
    })
}

pub(crate) fn resolve_structured_inclusive_join(
    process: &RawProcess,
    node: &RawNode,
) -> Result<Option<String>> {
    if node.gateway_kind != Some(BpmnGatewayKind::Inclusive) {
        return Ok(None);
    }

    let outgoing = process
        .flows
        .iter()
        .filter(|flow| flow.source_ref == node.bpmn_id)
        .collect::<Vec<_>>();
    let incoming_len = process
        .flows
        .iter()
        .filter(|flow| flow.target_ref == node.bpmn_id)
        .count();

    if !(incoming_len == 1 && outgoing.len() > 1) {
        return Ok(None);
    }

    let flow_by_source = process.flows.iter().fold(
        HashMap::<&str, Vec<&RawSequenceFlow>>::new(),
        |mut acc, flow| {
            acc.entry(flow.source_ref.as_str()).or_default().push(flow);
            acc
        },
    );
    let node_by_id = process
        .nodes
        .iter()
        .map(|raw_node| (raw_node.bpmn_id.as_str(), raw_node))
        .collect::<HashMap<_, _>>();

    let mut join_node_id = None::<String>;
    let mut seen_join_inputs = HashSet::new();
    for flow in outgoing {
        let (branch_join_node_id, join_input_flow_id) =
            trace_inclusive_branch_to_join(process, node, flow, &node_by_id, &flow_by_source)?;
        if !seen_join_inputs.insert(join_input_flow_id.clone()) {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "inclusive_split_branch_duplicate_join_input",
            });
        }
        match &join_node_id {
            Some(expected_join_node_id) if expected_join_node_id != &branch_join_node_id => {
                return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                    process_id: process.process_id.clone(),
                    node_id: node.bpmn_id.clone(),
                    detail: "inclusive_split_branch_mismatched_join",
                });
            }
            None => join_node_id = Some(branch_join_node_id),
            Some(_) => {}
        }
    }

    join_node_id
        .ok_or_else(|| BpmnEngineError::UnsupportedGatewayConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "inclusive_split_missing_join",
        })
        .map(Some)
}

fn trace_inclusive_branch_to_join<'a>(
    process: &RawProcess,
    split_node: &RawNode,
    initial_flow: &'a RawSequenceFlow,
    node_by_id: &HashMap<&'a str, &'a RawNode>,
    flow_by_source: &HashMap<&'a str, Vec<&'a RawSequenceFlow>>,
) -> Result<(String, String)> {
    let mut current_flow = initial_flow;
    let mut visited_nodes = HashSet::new();

    loop {
        let Some(current_node) = node_by_id.get(current_flow.target_ref.as_str()).copied() else {
            return Err(BpmnEngineError::UnknownSequenceFlowEndpoint {
                process_id: process.process_id.clone(),
                flow_id: current_flow.flow_id.clone(),
                endpoint: "target",
                node_id: current_flow.target_ref.clone(),
            });
        };

        if !visited_nodes.insert(current_node.bpmn_id.as_str()) {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: split_node.bpmn_id.clone(),
                detail: "inclusive_split_branch_not_linear",
            });
        }

        if current_node.gateway_kind == Some(BpmnGatewayKind::Inclusive) {
            let incoming_len = process
                .flows
                .iter()
                .filter(|flow| flow.target_ref == current_node.bpmn_id)
                .count();
            let outgoing_len = flow_by_source
                .get(current_node.bpmn_id.as_str())
                .map_or(0, Vec::len);
            if incoming_len > 1 && outgoing_len == 1 {
                return Ok((current_node.bpmn_id.clone(), current_flow.flow_id.clone()));
            }
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: split_node.bpmn_id.clone(),
                detail: "inclusive_split_branch_unsupported_gateway",
            });
        }

        if current_node.kind == BpmnNodeKind::Gateway {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: process.process_id.clone(),
                node_id: split_node.bpmn_id.clone(),
                detail: "inclusive_split_branch_unsupported_gateway",
            });
        }

        let outgoing = flow_by_source
            .get(current_node.bpmn_id.as_str())
            .cloned()
            .unwrap_or_default();
        match outgoing.as_slice() {
            [] => {
                return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                    process_id: process.process_id.clone(),
                    node_id: split_node.bpmn_id.clone(),
                    detail: "inclusive_split_branch_ends_before_join",
                });
            }
            [next_flow] => {
                current_flow = next_flow;
            }
            _ => {
                return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                    process_id: process.process_id.clone(),
                    node_id: split_node.bpmn_id.clone(),
                    detail: "inclusive_split_branch_not_linear",
                });
            }
        }
    }
}

pub(super) fn validate_standard_loops(process: &RawProcess) -> Result<()> {
    for node in &process.nodes {
        let Some(RawRepeatSpec::StandardLoop(loop_spec)) = &node.repeat else {
            continue;
        };

        if !matches!(
            node.kind,
            BpmnNodeKind::ServiceTask
                | BpmnNodeKind::ScriptTask
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

pub(super) fn validate_multi_instances(process: &RawProcess) -> Result<()> {
    for node in &process.nodes {
        let Some(shape) = multi_instance_validation_shape(node) else {
            continue;
        };

        if !matches!(
            node.kind,
            BpmnNodeKind::ServiceTask
                | BpmnNodeKind::ScriptTask
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

        validate_multi_instance_expansion(process, node, &shape)?;

        if let Some(completion_condition) = shape.completion_condition
            && !is_supported_multi_instance_completion_condition(completion_condition)
        {
            return Err(BpmnEngineError::UnsupportedLoopConfiguration {
                process_id: process.process_id.clone(),
                node_id: node.bpmn_id.clone(),
                detail: "unsupported_multi_instance_completion_condition_expression",
            });
        }
    }

    Ok(())
}

struct MultiInstanceValidationShape<'a> {
    loop_cardinality: Option<u32>,
    loop_data_input_ref: Option<&'a str>,
    input_data_item: Option<&'a str>,
    loop_data_output_ref: Option<&'a str>,
    output_data_item: Option<&'a str>,
    completion_condition: Option<&'a str>,
}

fn multi_instance_validation_shape(node: &RawNode) -> Option<MultiInstanceValidationShape<'_>> {
    match &node.repeat {
        Some(RawRepeatSpec::SequentialMultiInstance(multi_instance_spec)) => {
            Some(MultiInstanceValidationShape {
                loop_cardinality: multi_instance_spec.loop_cardinality,
                loop_data_input_ref: multi_instance_spec.loop_data_input_ref.as_deref(),
                input_data_item: multi_instance_spec.input_data_item.as_deref(),
                loop_data_output_ref: multi_instance_spec.loop_data_output_ref.as_deref(),
                output_data_item: multi_instance_spec.output_data_item.as_deref(),
                completion_condition: multi_instance_spec.completion_condition.as_deref(),
            })
        }
        Some(RawRepeatSpec::ParallelMultiInstance(multi_instance_spec)) => {
            Some(MultiInstanceValidationShape {
                loop_cardinality: multi_instance_spec.loop_cardinality,
                loop_data_input_ref: multi_instance_spec.loop_data_input_ref.as_deref(),
                input_data_item: multi_instance_spec.input_data_item.as_deref(),
                loop_data_output_ref: multi_instance_spec.loop_data_output_ref.as_deref(),
                output_data_item: multi_instance_spec.output_data_item.as_deref(),
                completion_condition: multi_instance_spec.completion_condition.as_deref(),
            })
        }
        _ => None,
    }
}

fn validate_multi_instance_expansion(
    process: &RawProcess,
    node: &RawNode,
    shape: &MultiInstanceValidationShape<'_>,
) -> Result<()> {
    if shape.loop_cardinality.is_some() && shape.loop_data_input_ref.is_some() {
        return Err(BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "mixed_multi_instance_expansion",
        });
    }
    if shape.loop_cardinality.is_none() && shape.loop_data_input_ref.is_none() {
        return Err(BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "missing_loop_cardinality_or_data_input",
        });
    }
    if shape.loop_data_input_ref.is_some() && shape.input_data_item.is_none() {
        return Err(BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "missing_input_data_item",
        });
    }
    if shape.loop_data_input_ref.is_none() && shape.input_data_item.is_some() {
        return Err(BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "missing_loop_data_input_ref",
        });
    }
    if shape.loop_data_output_ref.is_some() && shape.output_data_item.is_none() {
        return Err(BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "missing_output_data_item",
        });
    }
    if shape.loop_data_output_ref.is_none() && shape.output_data_item.is_some() {
        return Err(BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "missing_loop_data_output_ref",
        });
    }
    if let (Some(loop_data_input_ref), Some(loop_data_output_ref)) =
        (shape.loop_data_input_ref, shape.loop_data_output_ref)
        && loop_data_input_ref == loop_data_output_ref
    {
        return Err(BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: process.process_id.clone(),
            node_id: node.bpmn_id.clone(),
            detail: "unsupported_multi_instance_in_place_output",
        });
    }
    Ok(())
}

pub(super) fn validate_event_based_gateways(
    process: &RawProcess,
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
            if !node_ids.contains::<str>(target_id) {
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
                BpmnEventKind::Message
                    | BpmnEventKind::Signal
                    | BpmnEventKind::Timer
                    | BpmnEventKind::Conditional
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
