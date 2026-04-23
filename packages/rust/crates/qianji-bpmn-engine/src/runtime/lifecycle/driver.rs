use super::scope::{
    BpmnAdvanceOutcome, BpmnEngineError, BpmnFrontierExecutionProposal, BpmnFrontierExecutionStep,
    BpmnFrontierParallelJoinMerge, BpmnFrontierRuntimeAction, BpmnFrontierRuntimeBatch,
    BpmnHostBridge, BpmnInstanceState, BpmnNodeIndex, BpmnPackage, InstanceLifecycle,
    JoinRuntimeState, NodeRuntimeStatus, Result, plan_frontier_runtime_action,
    resolve_process_for_instance,
};
use super::{advance, call_activity, state};
use std::collections::HashMap;

pub(crate) async fn advance_instance_impl<H: BpmnHostBridge>(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    host: &H,
) -> Result<BpmnAdvanceOutcome> {
    match instance.lifecycle {
        InstanceLifecycle::Completed => return Ok(BpmnAdvanceOutcome::Completed),
        InstanceLifecycle::Failed => {
            return Ok(BpmnAdvanceOutcome::Failed(
                "instance lifecycle is failed".to_string(),
            ));
        }
        InstanceLifecycle::Suspended => {
            return Ok(BpmnAdvanceOutcome::Suspended(
                instance.suspend_reason.clone(),
            ));
        }
        InstanceLifecycle::Ready | InstanceLifecycle::Running | InstanceLifecycle::Waiting => {}
    }

    if instance.active_tokens.is_empty() {
        let process = resolve_process_for_instance(package, instance)?;
        if call_activity::can_bootstrap_start_token(instance) {
            call_activity::bootstrap_start_token(process, instance, host.now_unix_ms())?;
        } else {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "advance_instance_missing_frontier",
            });
        }
    }

    loop {
        let process = resolve_process_for_instance(package, instance)?;
        let frontier_action = plan_frontier_runtime_action(process, instance);
        match frontier_action {
            BpmnFrontierRuntimeAction::Execute(batch) => {
                if let Some(outcome) =
                    execute_frontier_runtime_batch(package, instance, host, &batch)?
                {
                    return Ok(outcome);
                }
            }
            BpmnFrontierRuntimeAction::BlockedOnHost(pending) => {
                instance.lifecycle = InstanceLifecycle::Waiting;
                return Ok(BpmnAdvanceOutcome::BlockedOnHost(pending));
            }
            BpmnFrontierRuntimeAction::WaitingExternalEvent => {
                instance.lifecycle = InstanceLifecycle::Waiting;
                return Ok(BpmnAdvanceOutcome::WaitingExternalEvent);
            }
            BpmnFrontierRuntimeAction::Suspended(reason) => {
                instance.lifecycle = InstanceLifecycle::Suspended;
                return Ok(BpmnAdvanceOutcome::Suspended(reason));
            }
            BpmnFrontierRuntimeAction::Stalled => {
                return Err(BpmnEngineError::UnsupportedOperation {
                    operation: "advance_instance_missing_runnable_token",
                });
            }
        }
    }
}

fn execute_frontier_runtime_batch<H: BpmnHostBridge>(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    host: &H,
    batch: &BpmnFrontierRuntimeBatch,
) -> Result<Option<BpmnAdvanceOutcome>> {
    let mut token_lookup = state::FrontierTokenLookup::default();
    match batch {
        BpmnFrontierRuntimeBatch::Proposals(proposals) => {
            execute_frontier_proposals(package, instance, host, proposals, &mut token_lookup)
        }
        BpmnFrontierRuntimeBatch::Steps(steps) => {
            execute_frontier_steps(package, instance, host, steps, &mut token_lookup)
        }
    }
}

fn execute_frontier_steps<H: BpmnHostBridge>(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    host: &H,
    steps: &[BpmnFrontierExecutionStep],
    token_lookup: &mut state::FrontierTokenLookup,
) -> Result<Option<BpmnAdvanceOutcome>> {
    for step in steps {
        match step {
            BpmnFrontierExecutionStep::Proposal(proposal) => {
                if let Some(outcome) =
                    execute_frontier_proposal(package, instance, host, proposal, token_lookup)?
                {
                    return Ok(Some(outcome));
                }
            }
            BpmnFrontierExecutionStep::ParallelJoin(group) => {
                if let Some(outcome) =
                    execute_parallel_join_merge(package, instance, host, group, token_lookup)?
                {
                    return Ok(Some(outcome));
                }
            }
        }
    }

    Ok(None)
}

fn execute_frontier_proposals<H: BpmnHostBridge>(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    host: &H,
    proposals: &[BpmnFrontierExecutionProposal],
    token_lookup: &mut state::FrontierTokenLookup,
) -> Result<Option<BpmnAdvanceOutcome>> {
    for proposal in proposals {
        if let Some(outcome) =
            execute_frontier_proposal(package, instance, host, proposal, token_lookup)?
        {
            return Ok(Some(outcome));
        }
    }
    Ok(None)
}

fn execute_frontier_proposal<H: BpmnHostBridge>(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    host: &H,
    proposal: &BpmnFrontierExecutionProposal,
    token_lookup: &mut state::FrontierTokenLookup,
) -> Result<Option<BpmnAdvanceOutcome>> {
    let Some(current_token_index) =
        token_lookup.resolve_frontier_proposal_token_index(instance, proposal)
    else {
        return Ok(None);
    };
    let process = resolve_process_for_instance(package, instance)?;
    let now_ms = host.now_unix_ms();
    let outcome = advance::advance_active_node(
        package,
        process,
        instance,
        current_token_index,
        proposal.node_index,
        now_ms,
    )?;
    token_lookup.invalidate();
    Ok(outcome)
}

fn execute_parallel_join_merge<H: BpmnHostBridge>(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    host: &H,
    group: &BpmnFrontierParallelJoinMerge,
    token_lookup: &mut state::FrontierTokenLookup,
) -> Result<Option<BpmnAdvanceOutcome>> {
    let process = resolve_process_for_instance(package, instance)?;
    let outgoing = process.outgoing_edge_indices(group.node_index);
    let incoming = process.incoming_edge_indices(group.node_index);
    if !parallel_join_merge_supported(instance, group.node_index, incoming.len()) {
        return execute_parallel_join_merge_fallback(
            package,
            instance,
            host,
            &group.proposals,
            token_lookup,
        );
    }

    if outgoing.len() != 1 {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "advance_instance_parallel_gateway_join_routing",
        });
    }

    let expected =
        u32::try_from(incoming.len()).map_err(|_| BpmnEngineError::UnsupportedOperation {
            operation: "advance_instance_parallel_gateway_join_incoming_overflow",
        })?;
    let incoming_edge_positions: HashMap<u32, usize> = incoming
        .iter()
        .enumerate()
        .map(|(position, edge_index)| (*edge_index, position))
        .collect();
    let mut buffered_counts =
        current_parallel_join_counts(instance, group.node_index, incoming.len());
    let mut buffered_token_ids = Vec::with_capacity(group.proposals.len());
    let mut activation_token_ids = Vec::new();

    for proposal in &group.proposals {
        let Some(current_token_index) =
            token_lookup.resolve_frontier_proposal_token_index(instance, proposal)
        else {
            continue;
        };
        let incoming_edge_index = instance
            .active_tokens
            .get(current_token_index)
            .and_then(|token| token.incoming_edge_index)
            .ok_or(BpmnEngineError::UnsupportedOperation {
                operation: "advance_instance_parallel_gateway_join_missing_arrival_edge",
            })?;
        let position = incoming_edge_positions
            .get(&incoming_edge_index)
            .copied()
            .ok_or(BpmnEngineError::UnsupportedOperation {
                operation: "advance_instance_parallel_gateway_join_unknown_arrival_edge",
            })?;
        buffered_counts[position] += 1;
        if buffered_counts.iter().all(|count| *count > 0) {
            for count in &mut buffered_counts {
                *count = count.saturating_sub(1);
            }
            activation_token_ids.push(proposal.token_id);
        } else {
            buffered_token_ids.push(proposal.token_id);
        }
    }

    store_parallel_join_counts(instance, group.node_index, expected, buffered_counts);
    let mut buffered_token_indices: Vec<usize> = buffered_token_ids
        .into_iter()
        .filter_map(|token_id| token_lookup.token_index_for_id(instance, token_id))
        .collect();
    buffered_token_indices.sort_unstable();
    buffered_token_indices.dedup();

    for token_index in buffered_token_indices.into_iter().rev() {
        let _ = state::remove_active_token(instance, token_index);
    }
    token_lookup.invalidate();

    if activation_token_ids.is_empty() {
        state::set_node_status(instance, group.node_index, NodeRuntimeStatus::Executing);
        if instance.active_tokens.is_empty() {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "advance_instance_parallel_gateway_join_missing_peer_token",
            });
        }
        state::record_transition(instance, host.now_unix_ms(), InstanceLifecycle::Running);
        return Ok(None);
    }

    state::set_node_status(instance, group.node_index, NodeRuntimeStatus::Completed);
    let outgoing_edge_index = outgoing[0];
    let next_node_index = process.edges[outgoing_edge_index as usize].to;
    for token_id in activation_token_ids {
        let Some(current_token_index) = token_lookup.token_index_for_id(instance, token_id) else {
            continue;
        };
        state::set_active_node_index(
            instance,
            current_token_index,
            outgoing_edge_index,
            next_node_index,
        );
    }
    state::set_node_status(instance, next_node_index, NodeRuntimeStatus::Queued);
    state::record_transition(instance, host.now_unix_ms(), InstanceLifecycle::Running);

    Ok(None)
}

fn execute_parallel_join_merge_fallback<H: BpmnHostBridge>(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    host: &H,
    proposals: &[BpmnFrontierExecutionProposal],
    token_lookup: &mut state::FrontierTokenLookup,
) -> Result<Option<BpmnAdvanceOutcome>> {
    execute_frontier_proposals(package, instance, host, proposals, token_lookup)
}

fn parallel_join_merge_supported(
    instance: &BpmnInstanceState,
    node_index: BpmnNodeIndex,
    incoming_len: usize,
) -> bool {
    instance
        .joins
        .iter()
        .find(|join| join.node_index == node_index)
        .is_none_or(|join| {
            join.incoming_counts.is_empty() || join.incoming_counts.len() == incoming_len
        })
}

fn current_parallel_join_counts(
    instance: &BpmnInstanceState,
    node_index: BpmnNodeIndex,
    incoming_len: usize,
) -> Vec<u32> {
    instance
        .joins
        .iter()
        .find(|join| join.node_index == node_index)
        .map_or_else(
            || vec![0; incoming_len],
            |join| {
                if join.incoming_counts.len() == incoming_len {
                    join.incoming_counts.clone()
                } else {
                    vec![0; incoming_len]
                }
            },
        )
}

fn store_parallel_join_counts(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
    expected: u32,
    buffered_counts: Vec<u32>,
) {
    let arrived = buffered_counts.iter().sum();
    if arrived == 0 {
        instance.joins.retain(|join| join.node_index != node_index);
        return;
    }

    if let Some(join) = instance
        .joins
        .iter_mut()
        .find(|join| join.node_index == node_index)
    {
        join.expected = expected;
        join.arrived = arrived;
        join.incoming_counts = buffered_counts;
        return;
    }

    instance.joins.push(JoinRuntimeState {
        node_index,
        activation_id: None,
        arrived,
        expected,
        incoming_counts: buffered_counts,
    });
}
