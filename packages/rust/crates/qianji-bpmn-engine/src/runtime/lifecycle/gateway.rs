use super::scope::{
    BpmnAdvanceOutcome, BpmnEngineError, BpmnGatewayKind, BpmnInstanceState, BpmnNodeIndex,
    BpmnProcessSpec, EventCompetitionState, InstanceLifecycle, NodeRuntimeStatus, Result,
    SuspendReason,
};
use super::{blocking, state};

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

    for edge_index in outgoing.iter().skip(1) {
        let target = process.edges[*edge_index as usize].to;
        state::push_active_token(instance, *edge_index, target);
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
    let edge_index = state::resolve_single_outgoing_edge(
        process,
        node_index,
        "advance_instance_exclusive_gateway_branching",
    )?;
    let next_node_index = process.edges[edge_index as usize].to;
    state::set_node_status(instance, node_index, NodeRuntimeStatus::Completed);
    state::set_active_node_index(instance, current_token_index, edge_index, next_node_index);
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
            process_id: process.key.process_id.to_string(),
            node_id: gateway.bpmn_id.to_string(),
            detail: "insufficient_outgoing_waits",
        });
    }

    let wait_node_indices = outgoing
        .iter()
        .map(|edge_index| process.edges[*edge_index as usize].to)
        .collect::<Vec<_>>();
    let Some(first_wait_node_index) = wait_node_indices.first().copied() else {
        return Err(BpmnEngineError::UnsupportedEventBasedGatewayConfiguration {
            process_id: process.key.process_id.to_string(),
            node_id: gateway.bpmn_id.to_string(),
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
    for (position, wait_node_index) in wait_node_indices.iter().copied().enumerate() {
        if position > 0 {
            state::push_active_token(instance, outgoing[position], wait_node_index);
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
