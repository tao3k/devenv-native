use super::scope::{
    BpmnAdvanceOutcome, BpmnEngineError, BpmnGatewayKind, BpmnInstanceState, BpmnNodeIndex,
    BpmnProcessSpec, EventCompetitionState, InclusiveJoinHint, InstanceLifecycle,
    NodeRuntimeStatus, Result, SuspendReason,
};
use super::{blocking, state};
use crate::repeat_condition::{GatewayConditionError, evaluate_gateway_condition};

pub(super) fn advance_gateway(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<Option<BpmnAdvanceOutcome>> {
    let gateway_kind = process.nodes[node_index as usize]
        .gateway_kind
        .as_ref()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "advance_instance_gateway_missing_kind",
        })?;

    match gateway_kind {
        BpmnGatewayKind::Parallel => {
            advance_parallel_gateway(process, instance, current_token_index, node_index, now_ms)?;
            Ok(None)
        }
        BpmnGatewayKind::Exclusive => {
            advance_exclusive_gateway(process, instance, current_token_index, node_index, now_ms)?;
            Ok(None)
        }
        BpmnGatewayKind::Inclusive => {
            advance_inclusive_gateway(process, instance, current_token_index, node_index, now_ms)?;
            Ok(None)
        }
        BpmnGatewayKind::EventBased => {
            advance_event_based_gateway(
                process,
                instance,
                current_token_index,
                node_index,
                now_ms,
            )?;
            Ok(None)
        }
    }
}

fn advance_parallel_gateway(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<()> {
    let incoming_len = process.incoming_edge_indices(node_index).len();
    let outgoing = process.outgoing_edge_indices(node_index);

    if incoming_len > 1 {
        return advance_parallel_join(
            process,
            instance,
            current_token_index,
            node_index,
            outgoing,
            now_ms,
        );
    }
    advance_parallel_split(
        process,
        instance,
        current_token_index,
        node_index,
        outgoing,
        now_ms,
    )
}

fn advance_parallel_split(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    outgoing: &[u32],
    now_ms: u64,
) -> Result<()> {
    let Some(first_edge_index) = outgoing.first() else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "advance_instance_parallel_gateway_missing_outgoing",
        });
    };

    state::set_node_status(instance, node_index, NodeRuntimeStatus::Completed);
    let first_target = process.edges[*first_edge_index as usize].to;
    state::set_active_node_index(
        instance,
        current_token_index,
        *first_edge_index,
        first_target,
    );
    state::set_node_status(instance, first_target, NodeRuntimeStatus::Queued);

    let mut token_ids = state::token_id_allocator(instance);
    for edge_index in outgoing.iter().skip(1) {
        let target = process.edges[*edge_index as usize].to;
        state::push_active_token_with_allocator(instance, *edge_index, target, &mut token_ids);
        state::set_node_status(instance, target, NodeRuntimeStatus::Queued);
    }

    state::record_transition(instance, now_ms, InstanceLifecycle::Running);
    Ok(())
}

fn advance_parallel_join(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    outgoing: &[u32],
    now_ms: u64,
) -> Result<()> {
    if outgoing.len() != 1 {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "advance_instance_parallel_gateway_join_routing",
        });
    }

    let incoming = process.incoming_edge_indices(node_index);
    let expected =
        u32::try_from(incoming.len()).map_err(|_| BpmnEngineError::UnsupportedOperation {
            operation: "advance_instance_parallel_gateway_join_incoming_overflow",
        })?;
    let incoming_edge_index = instance
        .active_tokens
        .get(current_token_index)
        .and_then(|token| token.incoming_edge_index)
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "advance_instance_parallel_gateway_join_missing_arrival_edge",
        })?;
    let ready = state::record_join_arrival(
        instance,
        node_index,
        expected,
        incoming,
        incoming_edge_index,
    )?;
    if !ready {
        state::set_node_status(instance, node_index, NodeRuntimeStatus::Executing);
        let _ = state::remove_active_token(instance, current_token_index);
        if instance.active_tokens.is_empty() {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "advance_instance_parallel_gateway_join_missing_peer_token",
            });
        }
        state::record_transition(instance, now_ms, InstanceLifecycle::Running);
        return Ok(());
    }

    state::consume_join_activation(instance, node_index, expected);
    state::set_node_status(instance, node_index, NodeRuntimeStatus::Completed);
    let outgoing_edge_index = outgoing[0];
    let next_node_index = process.edges[outgoing_edge_index as usize].to;
    state::set_active_node_index(
        instance,
        current_token_index,
        outgoing_edge_index,
        next_node_index,
    );
    state::set_node_status(instance, next_node_index, NodeRuntimeStatus::Queued);
    state::record_transition(instance, now_ms, InstanceLifecycle::Running);
    Ok(())
}

fn advance_exclusive_gateway(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<()> {
    let gateway = &process.nodes[node_index as usize];
    let outgoing = process.outgoing_edge_indices(node_index);
    let edge_index = if outgoing.len() <= 1 {
        state::resolve_single_outgoing_edge(
            process,
            node_index,
            "advance_instance_exclusive_gateway_branching",
        )?
    } else {
        select_exclusive_gateway_edge(process, instance, outgoing, gateway)?
    };
    let next_node_index = process.edges[edge_index as usize].to;
    state::set_node_status(instance, node_index, NodeRuntimeStatus::Completed);
    state::set_active_node_index(instance, current_token_index, edge_index, next_node_index);
    state::set_node_status(instance, next_node_index, NodeRuntimeStatus::Queued);
    state::record_transition(instance, now_ms, InstanceLifecycle::Running);
    Ok(())
}

fn advance_inclusive_gateway(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<()> {
    let incoming_len = process.incoming_edge_indices(node_index).len();
    let outgoing = process.outgoing_edge_indices(node_index);
    if incoming_len == 1 && outgoing.len() > 1 {
        return advance_inclusive_split(process, instance, current_token_index, node_index, now_ms);
    }
    if incoming_len > 1 && outgoing.len() == 1 {
        return advance_inclusive_join(process, instance, current_token_index, node_index, now_ms);
    }

    let gateway = &process.nodes[node_index as usize];
    Err(BpmnEngineError::UnsupportedGatewayConfiguration {
        process_id: (process.key.process_id.to_string()).into(),
        node_id: (gateway.bpmn_id.to_string()).into(),
        detail: "inclusive_gateway_requires_structured_split_or_join",
    })
}

fn select_exclusive_gateway_edge(
    process: &BpmnProcessSpec,
    instance: &BpmnInstanceState,
    outgoing: &[u32],
    gateway: &crate::ir_node_api::BpmnNodeSpec,
) -> Result<u32> {
    let default_edge_index = gateway.default_outgoing_edge;

    for edge_index in outgoing {
        if Some(*edge_index) == default_edge_index {
            continue;
        }
        let Some(condition_expression) = process.edges[*edge_index as usize]
            .condition_expression
            .as_deref()
        else {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: (process.key.process_id.to_string()).into(),
                node_id: (gateway.bpmn_id.to_string()).into(),
                detail: "missing_condition_expression",
            });
        };

        match evaluate_gateway_condition(condition_expression, &instance.variables) {
            Ok(true) => return Ok(*edge_index),
            Ok(false) => {}
            Err(GatewayConditionError::UnresolvedVariablePath(_)) => {
                return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                    process_id: (process.key.process_id.to_string()).into(),
                    node_id: (gateway.bpmn_id.to_string()).into(),
                    detail: "unresolved_condition_variable",
                });
            }
            Err(GatewayConditionError::UnsupportedExpression) => {
                return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                    process_id: (process.key.process_id.to_string()).into(),
                    node_id: (gateway.bpmn_id.to_string()).into(),
                    detail: "unsupported_condition_expression",
                });
            }
        }
    }

    default_edge_index.ok_or_else(|| BpmnEngineError::UnsupportedGatewayConfiguration {
        process_id: (process.key.process_id.to_string()).into(),
        node_id: (gateway.bpmn_id.to_string()).into(),
        detail: "no_matching_condition_or_default",
    })
}

fn advance_inclusive_split(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<()> {
    let gateway = &process.nodes[node_index as usize];
    let join_node_index = gateway.inclusive_join_node.ok_or_else(|| {
        BpmnEngineError::UnsupportedGatewayConfiguration {
            process_id: (process.key.process_id.to_string()).into(),
            node_id: (gateway.bpmn_id.to_string()).into(),
            detail: "inclusive_split_missing_join",
        }
    })?;
    let outgoing = process.outgoing_edge_indices(node_index);
    let selected = select_inclusive_gateway_edges(process, instance, outgoing, gateway)?;
    let expected_arrivals =
        u32::try_from(selected.len()).map_err(|_| BpmnEngineError::UnsupportedOperation {
            operation: "advance_instance_inclusive_gateway_selected_edge_count_overflow",
        })?;
    let activation_id = instance
        .active_tokens
        .get(current_token_index)
        .map(|token| token.token_id)
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "advance_instance_inclusive_gateway_missing_token",
        })?;
    let join_hint = InclusiveJoinHint {
        activation_id,
        join_node_index,
        expected_arrivals,
    };

    state::set_node_status(instance, node_index, NodeRuntimeStatus::Completed);
    let first_edge_index = selected[0];
    let first_target = process.edges[first_edge_index as usize].to;
    state::set_active_node_index(
        instance,
        current_token_index,
        first_edge_index,
        first_target,
    );
    state::set_token_inclusive_join_hint(instance, current_token_index, Some(join_hint.clone()));
    state::set_node_status(instance, first_target, NodeRuntimeStatus::Queued);

    let mut token_ids = state::token_id_allocator(instance);
    for edge_index in selected.iter().skip(1) {
        let target = process.edges[*edge_index as usize].to;
        state::push_active_token_with_join_hint_and_allocator(
            instance,
            *edge_index,
            target,
            join_hint.clone(),
            &mut token_ids,
        );
        state::set_node_status(instance, target, NodeRuntimeStatus::Queued);
    }

    state::record_transition(instance, now_ms, InstanceLifecycle::Running);
    Ok(())
}

fn select_inclusive_gateway_edges(
    process: &BpmnProcessSpec,
    instance: &BpmnInstanceState,
    outgoing: &[u32],
    gateway: &crate::ir_node_api::BpmnNodeSpec,
) -> Result<Vec<u32>> {
    let default_edge_index = gateway.default_outgoing_edge;
    let mut selected = outgoing
        .iter()
        .copied()
        .filter(|edge_index| Some(*edge_index) != default_edge_index)
        .map(|edge_index| evaluate_inclusive_gateway_edge(process, instance, gateway, edge_index))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    if selected.is_empty() {
        if let Some(default_edge_index) = default_edge_index {
            selected.push(default_edge_index);
            return Ok(selected);
        }
        return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
            process_id: (process.key.process_id.to_string()).into(),
            node_id: (gateway.bpmn_id.to_string()).into(),
            detail: "no_matching_condition_or_default",
        });
    }

    Ok(selected)
}

fn evaluate_inclusive_gateway_edge(
    process: &BpmnProcessSpec,
    instance: &BpmnInstanceState,
    gateway: &crate::ir_node_api::BpmnNodeSpec,
    edge_index: u32,
) -> Result<Option<u32>> {
    let Some(condition_expression) = process.edges[edge_index as usize]
        .condition_expression
        .as_deref()
    else {
        return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
            process_id: (process.key.process_id.to_string()).into(),
            node_id: (gateway.bpmn_id.to_string()).into(),
            detail: "missing_condition_expression",
        });
    };

    match evaluate_gateway_condition(condition_expression, &instance.variables) {
        Ok(true) => Ok(Some(edge_index)),
        Ok(false) => Ok(None),
        Err(GatewayConditionError::UnresolvedVariablePath(_)) => {
            Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: (process.key.process_id.to_string()).into(),
                node_id: (gateway.bpmn_id.to_string()).into(),
                detail: "unresolved_condition_variable",
            })
        }
        Err(GatewayConditionError::UnsupportedExpression) => {
            Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: (process.key.process_id.to_string()).into(),
                node_id: (gateway.bpmn_id.to_string()).into(),
                detail: "unsupported_condition_expression",
            })
        }
    }
}

fn advance_inclusive_join(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<()> {
    let gateway = &process.nodes[node_index as usize];
    let join_hint = instance
        .active_tokens
        .get(current_token_index)
        .and_then(|token| token.inclusive_join_hint.clone())
        .ok_or_else(|| BpmnEngineError::UnsupportedGatewayConfiguration {
            process_id: (process.key.process_id.to_string()).into(),
            node_id: (gateway.bpmn_id.to_string()).into(),
            detail: "inclusive_join_missing_activation_hint",
        })?;
    if join_hint.join_node_index != node_index {
        return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
            process_id: (process.key.process_id.to_string()).into(),
            node_id: (gateway.bpmn_id.to_string()).into(),
            detail: "inclusive_join_missing_activation_hint",
        });
    }

    let ready = state::record_scoped_join_arrival(
        instance,
        node_index,
        join_hint.activation_id,
        join_hint.expected_arrivals,
    );
    if !ready {
        state::set_node_status(instance, node_index, NodeRuntimeStatus::Executing);
        let _ = state::remove_active_token(instance, current_token_index);
        if instance.active_tokens.is_empty() {
            return Err(BpmnEngineError::UnsupportedGatewayConfiguration {
                process_id: (process.key.process_id.to_string()).into(),
                node_id: (gateway.bpmn_id.to_string()).into(),
                detail: "inclusive_join_missing_peer_token",
            });
        }
        state::record_transition(instance, now_ms, InstanceLifecycle::Running);
        return Ok(());
    }

    state::consume_scoped_join_activation(instance, node_index, join_hint.activation_id);
    state::set_node_status(instance, node_index, NodeRuntimeStatus::Completed);
    let outgoing_edge_index = state::resolve_single_outgoing_edge(
        process,
        node_index,
        "advance_instance_inclusive_gateway_join_routing",
    )?;
    let next_node_index = process.edges[outgoing_edge_index as usize].to;
    state::set_active_node_index(
        instance,
        current_token_index,
        outgoing_edge_index,
        next_node_index,
    );
    state::set_token_inclusive_join_hint(instance, current_token_index, None);
    state::set_node_status(instance, next_node_index, NodeRuntimeStatus::Queued);
    state::record_transition(instance, now_ms, InstanceLifecycle::Running);
    Ok(())
}

fn advance_event_based_gateway(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<()> {
    let gateway = &process.nodes[node_index as usize];
    let outgoing = process.outgoing_edge_indices(node_index);
    if outgoing.len() < 2 {
        return Err(BpmnEngineError::UnsupportedEventBasedGatewayConfiguration {
            process_id: (process.key.process_id.to_string()).into(),
            node_id: (gateway.bpmn_id.to_string()).into(),
            detail: "insufficient_outgoing_waits",
        });
    }

    let wait_node_indices = outgoing
        .iter()
        .map(|edge_index| process.edges[*edge_index as usize].to)
        .collect::<Vec<_>>();
    let Some(first_wait_node_index) = wait_node_indices.first().copied() else {
        return Err(BpmnEngineError::UnsupportedEventBasedGatewayConfiguration {
            process_id: (process.key.process_id.to_string()).into(),
            node_id: (gateway.bpmn_id.to_string()).into(),
            detail: "insufficient_outgoing_waits",
        });
    };

    state::set_node_status(instance, node_index, NodeRuntimeStatus::Completed);
    state::set_active_node_index(
        instance,
        current_token_index,
        outgoing[0],
        first_wait_node_index,
    );
    instance.waits.clear();
    let mut token_ids = state::token_id_allocator(instance);
    for (position, wait_node_index) in wait_node_indices.iter().copied().enumerate() {
        if position > 0 {
            state::push_active_token_with_allocator(
                instance,
                outgoing[position],
                wait_node_index,
                &mut token_ids,
            );
        }
        state::set_node_status(instance, wait_node_index, NodeRuntimeStatus::Executing);
        instance.waits.push(blocking::build_wait_registration(
            process,
            wait_node_index,
            None,
        )?);
    }
    instance.event_competition = Some(EventCompetitionState {
        gateway_node_index: node_index,
        wait_node_indices,
    });
    instance.suspend_reason = Some(SuspendReason::ExternalWait);
    state::record_transition(instance, now_ms, InstanceLifecycle::Waiting);

    Ok(())
}
