use crate::runtime::lifecycle::scope::{
    BpmnEngineError, BpmnInstanceState, BpmnNodeIndex, JoinRuntimeState, Result,
};

pub(crate) fn record_join_arrival(
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

pub(crate) fn consume_join_activation(
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
