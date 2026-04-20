use super::scope::*;
use super::{advance, call_activity, state};

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
        let frontier_plan = plan_frontier_step(process, instance);
        match frontier_plan.action {
            BpmnFrontierPlanAction::ExecuteBatch(batch) => {
                if let Some(outcome) = execute_frontier_batch(package, instance, host, &batch)? {
                    return Ok(outcome);
                }
            }
            BpmnFrontierPlanAction::BlockedOnHost(pending) => {
                instance.lifecycle = InstanceLifecycle::Waiting;
                return Ok(BpmnAdvanceOutcome::BlockedOnHost(pending));
            }
            BpmnFrontierPlanAction::WaitingExternalEvent => {
                instance.lifecycle = InstanceLifecycle::Waiting;
                return Ok(BpmnAdvanceOutcome::WaitingExternalEvent);
            }
            BpmnFrontierPlanAction::Suspended(reason) => {
                instance.lifecycle = InstanceLifecycle::Suspended;
                return Ok(BpmnAdvanceOutcome::Suspended(reason));
            }
            BpmnFrontierPlanAction::Stalled => {
                return Err(BpmnEngineError::UnsupportedOperation {
                    operation: "advance_instance_missing_runnable_token",
                });
            }
        }
    }
}

fn execute_frontier_batch<H: BpmnHostBridge>(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    host: &H,
    batch: &BpmnFrontierExecutionBatch,
) -> Result<Option<BpmnAdvanceOutcome>> {
    for step in &batch.steps {
        match step {
            BpmnFrontierExecutionStep::Proposal(proposal) => {
                if let Some(outcome) = execute_frontier_proposal(package, instance, host, proposal)?
                {
                    return Ok(Some(outcome));
                }
            }
            BpmnFrontierExecutionStep::ParallelJoin(group) => {
                if let Some(outcome) = execute_parallel_join_merge(package, instance, host, group)?
                {
                    return Ok(Some(outcome));
                }
            }
        }
    }

    Ok(None)
}

fn execute_frontier_proposal<H: BpmnHostBridge>(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    host: &H,
    proposal: &BpmnFrontierExecutionProposal,
) -> Result<Option<BpmnAdvanceOutcome>> {
    let Some(current_token_index) =
        state::resolve_frontier_proposal_token_index(instance, proposal)
    else {
        return Ok(None);
    };
    let process = resolve_process_for_instance(package, instance)?;
    let now_ms = host.now_unix_ms();
    advance::advance_active_node(
        package,
        process,
        instance,
        current_token_index,
        proposal.node_index,
        now_ms,
    )
}

fn execute_parallel_join_merge<H: BpmnHostBridge>(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    host: &H,
    group: &BpmnFrontierParallelJoinMerge,
) -> Result<Option<BpmnAdvanceOutcome>> {
    let process = resolve_process_for_instance(package, instance)?;
    let outgoing = process.outgoing_edge_indices(group.node_index);
    let incoming = process.incoming_edge_indices(group.node_index);
    if !parallel_join_merge_supported(instance, group.node_index, incoming.len()) {
        return execute_parallel_join_merge_fallback(package, instance, host, &group.proposals);
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
    let mut buffered_counts =
        current_parallel_join_counts(instance, group.node_index, incoming.len());
    let mut resolved_token_ids = Vec::new();
    let mut activation_token_ids = Vec::new();

    for proposal in &group.proposals {
        let Some(current_token_index) =
            state::resolve_frontier_proposal_token_index(instance, proposal)
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
        let position = incoming
            .iter()
            .position(|edge| *edge == incoming_edge_index)
            .ok_or(BpmnEngineError::UnsupportedOperation {
                operation: "advance_instance_parallel_gateway_join_unknown_arrival_edge",
            })?;
        buffered_counts[position] += 1;
        resolved_token_ids.push(proposal.token_id);
        if buffered_counts.iter().all(|count| *count > 0) {
            for count in &mut buffered_counts {
                *count = count.saturating_sub(1);
            }
            activation_token_ids.push(proposal.token_id);
        }
    }

    store_parallel_join_counts(instance, group.node_index, expected, buffered_counts);
    let buffered_token_ids: Vec<u64> = resolved_token_ids
        .iter()
        .copied()
        .filter(|token_id| !activation_token_ids.contains(token_id))
        .collect();

    for token_id in buffered_token_ids {
        if let Some(current_token_index) = state::token_index_for_id(instance, token_id) {
            let _ = state::remove_active_token(instance, current_token_index);
        }
    }

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
        let Some(current_token_index) = state::token_index_for_id(instance, token_id) else {
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
) -> Result<Option<BpmnAdvanceOutcome>> {
    for proposal in proposals {
        if let Some(outcome) = execute_frontier_proposal(package, instance, host, proposal)? {
            return Ok(Some(outcome));
        }
    }
    Ok(None)
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
        .map(|join| {
            if join.incoming_counts.len() == incoming_len {
                join.incoming_counts.clone()
            } else {
                vec![0; incoming_len]
            }
        })
        .unwrap_or_else(|| vec![0; incoming_len])
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
        arrived,
        expected,
        incoming_counts: buffered_counts,
    });
}
