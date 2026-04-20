use super::scope::*;

pub(super) fn find_single_start_node(process: &BpmnProcessSpec) -> Result<BpmnNodeIndex> {
    let mut start_nodes = process
        .nodes
        .iter()
        .filter(|node| node.kind == BpmnNodeKind::StartEvent)
        .map(|node| node.index);
    let Some(start_node_index) = start_nodes.next() else {
        return Err(BpmnEngineError::MissingRequiredProcessElement {
            process_id: process.key.process_id.to_string(),
            element: "start event",
        });
    };
    if start_nodes.next().is_some() {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "advance_instance_multiple_start_events",
        });
    }
    Ok(start_node_index)
}

pub(crate) fn set_active_node_index(
    instance: &mut BpmnInstanceState,
    token_index: usize,
    incoming_edge_index: u32,
    node_index: BpmnNodeIndex,
) {
    if let Some(token) = instance.active_tokens.get_mut(token_index) {
        token.node_index = node_index;
        token.incoming_edge_index = Some(incoming_edge_index);
    }
}

pub(super) fn remove_active_token(
    instance: &mut BpmnInstanceState,
    token_index: usize,
) -> Option<TokenRecord> {
    if token_index >= instance.active_tokens.len() {
        None
    } else {
        Some(instance.active_tokens.remove(token_index))
    }
}

pub(super) fn push_active_token(
    instance: &mut BpmnInstanceState,
    incoming_edge_index: u32,
    node_index: BpmnNodeIndex,
) {
    let _ = push_active_token_with_arrival(instance, Some(incoming_edge_index), node_index);
}

pub(super) fn push_active_token_with_arrival(
    instance: &mut BpmnInstanceState,
    incoming_edge_index: Option<u32>,
    node_index: BpmnNodeIndex,
) -> u64 {
    let token_id = next_token_id(instance);
    instance.active_tokens.push(TokenRecord {
        token_id,
        node_index,
        incoming_edge_index,
    });
    token_id
}

pub(crate) fn resolve_single_outgoing_edge(
    process: &BpmnProcessSpec,
    node_index: BpmnNodeIndex,
    operation: &'static str,
) -> Result<u32> {
    let outgoing = process.outgoing_edge_indices(node_index);
    if outgoing.len() != 1 {
        return Err(BpmnEngineError::UnsupportedOperation { operation });
    }
    Ok(outgoing[0])
}

pub(crate) fn set_node_status(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
    status: NodeRuntimeStatus,
) {
    if let Some(node_state) = instance.node_states.get_mut(node_index as usize) {
        node_state.status = status;
    }
}

pub(crate) fn record_transition(
    instance: &mut BpmnInstanceState,
    now_ms: u64,
    lifecycle: InstanceLifecycle,
) {
    instance.sequence += 1;
    instance.lifecycle = lifecycle;
    instance.updated_at_ms = now_ms;
}

pub(super) fn resolve_frontier_proposal_token_index(
    instance: &BpmnInstanceState,
    proposal: &BpmnFrontierExecutionProposal,
) -> Option<usize> {
    let token_index = token_index_for_id(instance, proposal.token_id)?;
    let token = instance.active_tokens.get(token_index)?;
    (token.node_index == proposal.node_index
        && token.incoming_edge_index == proposal.incoming_edge_index)
        .then_some(token_index)
}

pub(super) fn token_index_for_id(instance: &BpmnInstanceState, token_id: u64) -> Option<usize> {
    instance
        .active_tokens
        .iter()
        .position(|token| token.token_id == token_id)
}

pub(super) fn token_index_for_node(
    instance: &BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> Option<usize> {
    instance
        .active_tokens
        .iter()
        .position(|token| token.node_index == node_index)
}

pub(super) fn clear_pending_host_work(instance: &mut BpmnInstanceState, token_id: u64) {
    instance
        .pending_host_work
        .retain(|pending| pending.token_id != token_id);
}

pub(super) fn has_pending_host_work_for_node(
    instance: &BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> bool {
    instance
        .pending_host_work
        .iter()
        .any(|pending| pending.node_index == node_index)
}

pub(super) fn clear_boundary_wait_for_node(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
) {
    instance
        .waits
        .retain(|wait| wait.blocking_node_index != Some(node_index));
}

pub(super) fn next_token_id(instance: &BpmnInstanceState) -> u64 {
    instance
        .active_tokens
        .iter()
        .map(|token| token.token_id)
        .max()
        .unwrap_or(instance.sequence)
        .max(instance.sequence)
        + 1
}

pub(super) fn record_join_arrival(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
    expected: u32,
    incoming: &[u32],
    incoming_edge_index: u32,
) -> Result<bool> {
    if let Some(join) = instance
        .joins
        .iter_mut()
        .find(|join| join.node_index == node_index)
    {
        join.expected = expected;
        if join.incoming_counts.len() == incoming.len() {
            let position = incoming
                .iter()
                .position(|edge| *edge == incoming_edge_index)
                .ok_or(BpmnEngineError::UnsupportedOperation {
                    operation: "advance_instance_parallel_gateway_join_unknown_arrival_edge",
                })?;
            join.incoming_counts[position] += 1;
            join.arrived += 1;
            return Ok(join.incoming_counts.iter().all(|count| *count > 0));
        }

        // Preserve legacy aggregate behavior when older checkpoints do not yet
        // carry per-edge buffered arrival counts.
        join.arrived += 1;
        return Ok(join.arrived >= expected);
    }

    let mut incoming_counts = vec![0; incoming.len()];
    let position = incoming
        .iter()
        .position(|edge| *edge == incoming_edge_index)
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "advance_instance_parallel_gateway_join_unknown_arrival_edge",
        })?;
    incoming_counts[position] = 1;
    instance.joins.push(JoinRuntimeState {
        node_index,
        arrived: 1,
        expected,
        incoming_counts,
    });
    Ok(false)
}

fn clear_join_state(instance: &mut BpmnInstanceState, node_index: BpmnNodeIndex) {
    instance.joins.retain(|join| join.node_index != node_index);
}

pub(super) fn consume_join_activation(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
    expected: u32,
) {
    let mut should_clear = false;
    let expected_len = usize::try_from(expected).ok();

    if let Some(join) = instance
        .joins
        .iter_mut()
        .find(|join| join.node_index == node_index)
    {
        if expected_len.is_some_and(|expected_len| join.incoming_counts.len() == expected_len) {
            for count in &mut join.incoming_counts {
                *count = count.saturating_sub(1);
            }
            join.arrived = join.incoming_counts.iter().sum();
        } else {
            join.arrived = join.arrived.saturating_sub(expected);
        }
        should_clear = join.arrived == 0;
    }

    if should_clear {
        clear_join_state(instance, node_index);
    }
}
