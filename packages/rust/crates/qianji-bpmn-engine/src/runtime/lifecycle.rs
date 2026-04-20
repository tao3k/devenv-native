//! Runtime advancement outcome shells.

use super::{
    BpmnFrontierExecutionBatch, BpmnFrontierExecutionProposal, BpmnFrontierExecutionStep,
    BpmnFrontierParallelJoinMerge, BpmnFrontierPlanAction, BpmnInstanceState,
    EventCompetitionState, InstanceLifecycle, JoinRuntimeState, NodeRuntimeStatus, PendingHostWork,
    PendingHostWorkKind, SuspendReason, TokenRecord, WaitKind, WaitRegistration,
    clear_sequential_multi_instance_state, clear_standard_loop_state,
    ensure_sequential_multi_instance_state, ensure_standard_loop_state,
    increment_sequential_multi_instance_iterations, increment_standard_loop_iterations,
    install_process_state, plan_frontier_step, pop_call_activity_frame, push_call_activity_frame,
    resolve_process_for_instance, restore_call_activity_frame, standard_loop_completed_iterations,
};
use crate::dmn::{DmnEvaluationRequest, evaluate_dmn_decision_sync};
use crate::error::{BpmnEngineError, Result};
use crate::host::{BpmnHostBridge, PendingHostWorkResult};
use crate::ir::{
    BpmnEventKind, BpmnGatewayKind, BpmnNodeIndex, BpmnNodeKind, BpmnPackage, BpmnProcessSpec,
    BpmnRepeatSpec, BpmnStandardLoopSpec,
};
use std::borrow::Borrow;

/// High-level outcome from one runtime advance attempt.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BpmnAdvanceOutcome {
    /// Internal state progressed without blocking.
    Advanced,
    /// Blocked on host-dispatched work.
    BlockedOnHost(Vec<PendingHostWork>),
    /// Waiting on an external event or user/system signal.
    WaitingExternalEvent,
    /// Suspended intentionally with an optional reason.
    Suspended(Option<SuspendReason>),
    /// Completed successfully.
    Completed,
    /// Failed terminally with a message.
    Failed(String),
}

/// Advances one BPMN instance within the bounded runtime subset.
///
/// # Errors
///
/// Returns [`BpmnEngineError`] when the target process cannot be found or when
/// the current instance/model shape exceeds the supported bounded subset.
pub async fn advance_instance<H: BpmnHostBridge>(
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
        if can_bootstrap_start_token(instance) {
            bootstrap_start_token(process, instance, host.now_unix_ms())?;
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
    let Some(current_token_index) = resolve_frontier_proposal_token_index(instance, proposal)
    else {
        return Ok(None);
    };
    let process = resolve_process_for_instance(package, instance)?;
    let now_ms = host.now_unix_ms();
    advance_active_node(
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
        let Some(current_token_index) = resolve_frontier_proposal_token_index(instance, proposal)
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

    if resolved_token_ids.is_empty() {
        return Ok(None);
    }

    let activation_token_ids_set = activation_token_ids
        .iter()
        .copied()
        .collect::<std::collections::HashSet<_>>();
    for token_id in &resolved_token_ids {
        if activation_token_ids_set.contains(token_id) {
            continue;
        }
        if let Some(token_index) = token_index_for_id(instance, *token_id) {
            let _ = remove_active_token(instance, token_index);
        }
    }
    store_parallel_join_counts(instance, group.node_index, expected, buffered_counts);

    let now_ms = host.now_unix_ms();
    if activation_token_ids.is_empty() {
        set_node_status(instance, group.node_index, NodeRuntimeStatus::Executing);
        if instance.active_tokens.is_empty() {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "advance_instance_parallel_gateway_join_missing_peer_token",
            });
        }
        record_transition(instance, now_ms, InstanceLifecycle::Running);
        return Ok(None);
    }

    let outgoing_edge_index = outgoing[0];
    let next_node_index = process.edges[outgoing_edge_index as usize].to;
    set_node_status(instance, group.node_index, NodeRuntimeStatus::Completed);
    for token_id in activation_token_ids {
        let token_index = token_index_for_id(instance, token_id).ok_or(
            BpmnEngineError::UnsupportedOperation {
                operation: "advance_instance_parallel_gateway_join_missing_activation_token",
            },
        )?;
        set_active_node_index(instance, token_index, outgoing_edge_index, next_node_index);
    }
    set_node_status(instance, next_node_index, NodeRuntimeStatus::Queued);
    record_transition(instance, now_ms, InstanceLifecycle::Running);
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
        .is_none_or(|join| join.incoming_counts.len() == incoming_len)
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
            |join| join.incoming_counts.clone(),
        )
}

fn store_parallel_join_counts(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
    expected: u32,
    incoming_counts: Vec<u32>,
) {
    let arrived = incoming_counts.iter().sum();
    if arrived == 0 {
        clear_join_state(instance, node_index);
        return;
    }

    if let Some(join) = instance
        .joins
        .iter_mut()
        .find(|join| join.node_index == node_index)
    {
        join.expected = expected;
        join.arrived = arrived;
        join.incoming_counts = incoming_counts;
        return;
    }

    instance.joins.push(JoinRuntimeState {
        node_index,
        arrived,
        expected,
        incoming_counts,
    });
}

fn advance_active_node(
    package: &BpmnPackage,
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<Option<BpmnAdvanceOutcome>> {
    let current_node = &process.nodes[current_node_index as usize];

    match current_node.kind {
        BpmnNodeKind::StartEvent => {
            advance_start_event(
                process,
                instance,
                current_token_index,
                current_node_index,
                now_ms,
            )?;
            Ok(None)
        }
        BpmnNodeKind::EndEvent => advance_end_event(
            package,
            instance,
            current_token_index,
            process,
            current_node_index,
            now_ms,
        ),
        BpmnNodeKind::IntermediateCatchEvent => {
            register_intermediate_wait(process, instance, current_node_index, now_ms)?;
            Ok(None)
        }
        BpmnNodeKind::BoundaryEvent => Err(BpmnEngineError::UnsupportedOperation {
            operation: "advance_instance_boundary_event_direct_execution",
        }),
        BpmnNodeKind::ServiceTask => advance_host_task_node(
            process,
            instance,
            current_token_index,
            current_node_index,
            PendingHostWorkKind::Service,
            now_ms,
        ),
        BpmnNodeKind::UserTask => advance_host_task_node(
            process,
            instance,
            current_token_index,
            current_node_index,
            PendingHostWorkKind::User,
            now_ms,
        ),
        BpmnNodeKind::ManualTask => advance_host_task_node(
            process,
            instance,
            current_token_index,
            current_node_index,
            PendingHostWorkKind::Manual,
            now_ms,
        ),
        BpmnNodeKind::BusinessRuleTask => advance_business_rule_task(
            package,
            process,
            instance,
            current_token_index,
            current_node,
            current_node_index,
            now_ms,
        ),
        BpmnNodeKind::Gateway => advance_gateway(
            process,
            instance,
            current_token_index,
            current_node_index,
            now_ms,
        ),
        BpmnNodeKind::SubProcess => {
            enter_call_activity(
                package,
                instance,
                current_token_index,
                process,
                current_node_index,
                now_ms,
            )?;
            Ok(None)
        }
    }
}

fn advance_start_event(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<()> {
    let edge_index = resolve_single_outgoing_edge(
        process,
        current_node_index,
        "advance_instance_start_event_routing",
    )?;
    let next_node_index = process.edges[edge_index as usize].to;
    set_node_status(instance, current_node_index, NodeRuntimeStatus::Completed);
    set_active_node_index(instance, current_token_index, edge_index, next_node_index);
    set_node_status(instance, next_node_index, NodeRuntimeStatus::Queued);
    record_transition(instance, now_ms, InstanceLifecycle::Running);
    Ok(())
}

fn advance_end_event(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    _process: &BpmnProcessSpec,
    current_node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<Option<BpmnAdvanceOutcome>> {
    set_node_status(instance, current_node_index, NodeRuntimeStatus::Completed);
    let _ = remove_active_token(instance, current_token_index);
    if !instance.active_tokens.is_empty() {
        record_transition(instance, now_ms, InstanceLifecycle::Running);
        return Ok(None);
    }
    if instance.call_stack.is_empty() {
        instance.pending_host_work.clear();
        instance.waits.clear();
        instance.suspend_reason = None;
        record_transition(instance, now_ms, InstanceLifecycle::Completed);
        return Ok(Some(BpmnAdvanceOutcome::Completed));
    }

    complete_call_activity(package, instance, now_ms)?;
    Ok(None)
}

fn advance_host_task_node(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    kind: PendingHostWorkKind,
    now_ms: u64,
) -> Result<Option<BpmnAdvanceOutcome>> {
    if prepare_standard_loop_iteration(
        process,
        instance,
        current_token_index,
        current_node_index,
        now_ms,
    )? || prepare_sequential_multi_instance_iteration(
        process,
        instance,
        current_token_index,
        current_node_index,
        now_ms,
    )? {
        return Ok(None);
    }
    block_on_host_work(
        process,
        instance,
        current_token_index,
        current_node_index,
        kind,
        now_ms,
    )?;
    Ok(None)
}

fn advance_business_rule_task(
    package: &BpmnPackage,
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node: &super::super::ir::BpmnNodeSpec,
    current_node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<Option<BpmnAdvanceOutcome>> {
    if prepare_standard_loop_iteration(
        process,
        instance,
        current_token_index,
        current_node_index,
        now_ms,
    )? || prepare_sequential_multi_instance_iteration(
        process,
        instance,
        current_token_index,
        current_node_index,
        now_ms,
    )? {
        return Ok(None);
    }
    let decision = current_node.decision.clone().ok_or_else(|| {
        BpmnEngineError::MissingBusinessRuleDecisionRef {
            process_id: process.key.process_id.to_string(),
            node_id: current_node.bpmn_id.to_string(),
        }
    })?;
    if let Some(definition) = package.find_dmn_decision(&decision)? {
        let evaluation = evaluate_dmn_decision_sync(
            definition,
            &DmnEvaluationRequest::new(decision.clone(), instance.variables.clone()),
        )?;
        complete_local_task_execution(
            process,
            instance,
            current_token_index,
            current_node_index,
            &evaluation.output,
            now_ms,
        )?;
        return Ok(None);
    }
    block_on_business_rule_work(
        process,
        instance,
        current_token_index,
        current_node_index,
        decision,
        now_ms,
    )?;
    Ok(None)
}

/// Applies one host-side completion result to the currently blocked BPMN
/// instance and resumes local routing state.
///
/// # Errors
///
/// Returns [`BpmnEngineError::MissingPendingHostWork`] when the instance is not
/// currently blocked on host work, [`BpmnEngineError::HostResultKindMismatch`]
/// when the supplied host result does not match the pending work kind, or
/// [`BpmnEngineError`] when the process/model shape exceeds the supported
/// bounded subset.
pub fn apply_pending_host_work_result(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    token_id: u64,
    result: impl Borrow<PendingHostWorkResult>,
    completed_at_ms: u64,
) -> Result<BpmnAdvanceOutcome> {
    let result = result.borrow();
    let pending = instance
        .pending_host_work
        .iter()
        .find(|pending| pending.token_id == token_id)
        .cloned()
        .ok_or_else(|| {
            if instance.pending_host_work.is_empty() {
                return BpmnEngineError::MissingPendingHostWork {
                    instance_id: instance.instance_id.to_string(),
                };
            }
            BpmnEngineError::MissingPendingHostWorkToken {
                instance_id: instance.instance_id.to_string(),
                token_id,
            }
        })?;
    let process = resolve_process_for_instance(package, instance)?;

    if pending.kind != result.kind() {
        return Err(BpmnEngineError::HostResultKindMismatch {
            node_index: pending.node_index,
            expected: pending_host_kind_name(&pending.kind),
            actual: result.kind_name(),
        });
    }

    let current_node = &process.nodes[pending.node_index as usize];
    if !node_matches_pending_kind(&current_node.kind, &pending.kind) {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "apply_pending_host_work_result_node_kind_mismatch",
        });
    }

    let token_index =
        token_index_for_id(instance, token_id).ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "apply_pending_host_work_result_missing_active_token",
        })?;
    clear_pending_host_work(instance, token_id);
    clear_boundary_wait_for_node(instance, pending.node_index);
    if instance.pending_host_work.is_empty() && instance.waits.is_empty() {
        instance.suspend_reason = None;
    }
    complete_local_task_execution(
        process,
        instance,
        token_index,
        pending.node_index,
        result.data(),
        completed_at_ms,
    )?;

    Ok(BpmnAdvanceOutcome::Advanced)
}

fn can_bootstrap_start_token(instance: &BpmnInstanceState) -> bool {
    instance.sequence == 0
        && matches!(instance.lifecycle, InstanceLifecycle::Ready)
        && instance
            .node_states
            .iter()
            .all(|state| state.status == NodeRuntimeStatus::Idle)
}

fn register_intermediate_wait(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<()> {
    set_node_status(instance, node_index, NodeRuntimeStatus::Executing);
    instance
        .waits
        .retain(|wait| wait.node_index != node_index || wait.blocking_node_index.is_some());
    instance
        .waits
        .push(build_wait_registration(process, node_index, None)?);
    instance.suspend_reason = Some(SuspendReason::ExternalWait);
    record_transition(instance, now_ms, InstanceLifecycle::Waiting);

    Ok(())
}

fn bootstrap_start_token(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    now_ms: u64,
) -> Result<()> {
    let start_node_index = find_single_start_node(process)?;
    instance.active_tokens.push(TokenRecord {
        token_id: instance.sequence + 1,
        node_index: start_node_index,
        incoming_edge_index: None,
    });
    set_node_status(instance, start_node_index, NodeRuntimeStatus::Queued);
    record_transition(instance, now_ms, InstanceLifecycle::Running);
    Ok(())
}

fn enter_call_activity(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    process: &BpmnProcessSpec,
    node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<()> {
    let node = &process.nodes[node_index as usize];
    let called_process_id = node.called_process_id.as_ref().ok_or_else(|| {
        BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: process.key.process_id.to_string(),
            node_id: node.bpmn_id.to_string(),
            detail: "embedded_subprocess",
        }
    })?;

    if called_process_id.as_ref() == process.key.process_id.as_ref()
        || instance
            .call_stack
            .iter()
            .any(|frame| frame.process.process_id.as_ref() == called_process_id.as_ref())
    {
        return Err(BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: process.key.process_id.to_string(),
            node_id: node.bpmn_id.to_string(),
            detail: "recursive_call_activity",
        });
    }

    let (called_process_index, called_process) = package
        .find_process_position(called_process_id.as_ref())
        .ok_or_else(|| BpmnEngineError::UnknownCalledProcess {
            process_id: process.key.process_id.to_string(),
            node_id: node.bpmn_id.to_string(),
            called_process_id: called_process_id.to_string(),
        })?;

    set_node_status(instance, node_index, NodeRuntimeStatus::Executing);
    push_call_activity_frame(instance, node_index);
    install_process_state(instance, called_process, called_process_index);
    let _ = remove_active_token(instance, current_token_index);
    bootstrap_start_token(called_process, instance, now_ms)?;
    Ok(())
}

fn complete_call_activity(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    now_ms: u64,
) -> Result<()> {
    let frame = pop_call_activity_frame(instance).ok_or(BpmnEngineError::UnsupportedOperation {
        operation: "complete_call_activity_missing_parent_frame",
    })?;
    let return_node_index = restore_call_activity_frame(instance, frame);
    let process = resolve_process_for_instance(package, instance)?;

    complete_node_and_route(
        process,
        instance,
        token_index_for_node(instance, return_node_index).ok_or(
            BpmnEngineError::UnsupportedOperation {
                operation: "complete_call_activity_missing_parent_token",
            },
        )?,
        return_node_index,
        now_ms,
        "complete_call_activity_routing",
    )?;
    Ok(())
}

fn advance_gateway(
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

    set_node_status(instance, node_index, NodeRuntimeStatus::Completed);
    let first_target = process.edges[*first_edge_index as usize].to;
    set_active_node_index(
        instance,
        current_token_index,
        *first_edge_index,
        first_target,
    );
    set_node_status(instance, first_target, NodeRuntimeStatus::Queued);

    for edge_index in outgoing.iter().skip(1) {
        let target = process.edges[*edge_index as usize].to;
        push_active_token(instance, *edge_index, target);
        set_node_status(instance, target, NodeRuntimeStatus::Queued);
    }

    record_transition(instance, now_ms, InstanceLifecycle::Running);
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
    let ready = record_join_arrival(
        instance,
        node_index,
        expected,
        incoming,
        incoming_edge_index,
    )?;
    if !ready {
        set_node_status(instance, node_index, NodeRuntimeStatus::Executing);
        let _ = remove_active_token(instance, current_token_index);
        if instance.active_tokens.is_empty() {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "advance_instance_parallel_gateway_join_missing_peer_token",
            });
        }
        record_transition(instance, now_ms, InstanceLifecycle::Running);
        return Ok(());
    }

    consume_join_activation(instance, node_index, expected);
    set_node_status(instance, node_index, NodeRuntimeStatus::Completed);
    let outgoing_edge_index = outgoing[0];
    let next_node_index = process.edges[outgoing_edge_index as usize].to;
    set_active_node_index(
        instance,
        current_token_index,
        outgoing_edge_index,
        next_node_index,
    );
    set_node_status(instance, next_node_index, NodeRuntimeStatus::Queued);
    record_transition(instance, now_ms, InstanceLifecycle::Running);
    Ok(())
}

fn advance_exclusive_gateway(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<()> {
    let edge_index = resolve_single_outgoing_edge(
        process,
        node_index,
        "advance_instance_exclusive_gateway_branching",
    )?;
    let next_node_index = process.edges[edge_index as usize].to;
    set_node_status(instance, node_index, NodeRuntimeStatus::Completed);
    set_active_node_index(instance, current_token_index, edge_index, next_node_index);
    set_node_status(instance, next_node_index, NodeRuntimeStatus::Queued);
    record_transition(instance, now_ms, InstanceLifecycle::Running);
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

    set_node_status(instance, node_index, NodeRuntimeStatus::Completed);
    set_active_node_index(
        instance,
        current_token_index,
        outgoing[0],
        first_wait_node_index,
    );
    instance.waits.clear();
    for (position, wait_node_index) in wait_node_indices.iter().copied().enumerate() {
        if position > 0 {
            push_active_token(instance, outgoing[position], wait_node_index);
        }
        set_node_status(instance, wait_node_index, NodeRuntimeStatus::Executing);
        instance
            .waits
            .push(build_wait_registration(process, wait_node_index, None)?);
    }
    instance.event_competition = Some(EventCompetitionState {
        gateway_node_index: node_index,
        wait_node_indices,
    });
    instance.suspend_reason = Some(SuspendReason::ExternalWait);
    record_transition(instance, now_ms, InstanceLifecycle::Waiting);

    Ok(())
}

fn prepare_standard_loop_iteration(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<bool> {
    let Some(BpmnRepeatSpec::StandardLoop(loop_spec)) =
        process.nodes[node_index as usize].repeat.as_ref()
    else {
        return Ok(false);
    };

    let completed_iterations = standard_loop_completed_iterations(instance, node_index);
    if loop_spec.test_before
        && !standard_loop_should_continue(
            process,
            node_index,
            loop_spec,
            completed_iterations,
            &instance.variables,
        )?
    {
        clear_standard_loop_state(instance, node_index);
        complete_node_and_route(
            process,
            instance,
            current_token_index,
            node_index,
            now_ms,
            "advance_instance_standard_loop_routing",
        )?;
        return Ok(true);
    }

    ensure_standard_loop_state(instance, node_index);
    Ok(false)
}

fn prepare_sequential_multi_instance_iteration(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<bool> {
    let Some(BpmnRepeatSpec::SequentialMultiInstance(loop_spec)) =
        process.nodes[node_index as usize].repeat.as_ref()
    else {
        return Ok(false);
    };

    if loop_spec.loop_cardinality == 0 {
        clear_sequential_multi_instance_state(instance, node_index);
        complete_node_and_route(
            process,
            instance,
            current_token_index,
            node_index,
            now_ms,
            "advance_instance_sequential_multi_instance_routing",
        )?;
        return Ok(true);
    }

    ensure_sequential_multi_instance_state(instance, node_index, loop_spec.loop_cardinality);
    Ok(false)
}

fn find_single_start_node(process: &BpmnProcessSpec) -> Result<BpmnNodeIndex> {
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

pub(super) fn set_active_node_index(
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
    let token_id = next_token_id(instance);
    instance.active_tokens.push(TokenRecord {
        token_id,
        node_index,
        incoming_edge_index: Some(incoming_edge_index),
    });
}

pub(super) fn resolve_single_outgoing_edge(
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

pub(super) fn set_node_status(
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
    status: NodeRuntimeStatus,
) {
    if let Some(node_state) = instance.node_states.get_mut(node_index as usize) {
        node_state.status = status;
    }
}

pub(super) fn record_transition(
    instance: &mut BpmnInstanceState,
    now_ms: u64,
    lifecycle: InstanceLifecycle,
) {
    instance.sequence += 1;
    instance.lifecycle = lifecycle;
    instance.updated_at_ms = now_ms;
}

fn complete_node_and_route(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    now_ms: u64,
    operation: &'static str,
) -> Result<()> {
    set_node_status(instance, node_index, NodeRuntimeStatus::Completed);
    let edge_index = resolve_single_outgoing_edge(process, node_index, operation)?;
    let next_node_index = process.edges[edge_index as usize].to;
    set_active_node_index(instance, current_token_index, edge_index, next_node_index);
    set_node_status(instance, next_node_index, NodeRuntimeStatus::Queued);
    record_transition(instance, now_ms, InstanceLifecycle::Running);
    Ok(())
}

fn complete_local_task_execution(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    output_data: &serde_json::Value,
    now_ms: u64,
) -> Result<()> {
    merge_output_data(&mut instance.variables, output_data);

    if let Some(BpmnRepeatSpec::StandardLoop(loop_spec)) =
        process.nodes[node_index as usize].repeat.as_ref()
    {
        let completed_iterations = increment_standard_loop_iterations(instance, node_index);
        if standard_loop_should_continue(
            process,
            node_index,
            loop_spec,
            completed_iterations,
            &instance.variables,
        )? {
            set_node_status(instance, node_index, NodeRuntimeStatus::Queued);
            record_transition(instance, now_ms, InstanceLifecycle::Running);
            return Ok(());
        }
        clear_standard_loop_state(instance, node_index);
    }

    if let Some(BpmnRepeatSpec::SequentialMultiInstance(loop_spec)) =
        process.nodes[node_index as usize].repeat.as_ref()
    {
        ensure_sequential_multi_instance_state(instance, node_index, loop_spec.loop_cardinality);
        let Some((completed_iterations, total_iterations)) =
            increment_sequential_multi_instance_iterations(instance, node_index)
        else {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "complete_local_task_execution_missing_multi_instance_state",
            });
        };
        if completed_iterations < total_iterations {
            set_node_status(instance, node_index, NodeRuntimeStatus::Queued);
            record_transition(instance, now_ms, InstanceLifecycle::Running);
            return Ok(());
        }
        clear_sequential_multi_instance_state(instance, node_index);
    }

    complete_node_and_route(
        process,
        instance,
        current_token_index,
        node_index,
        now_ms,
        "complete_local_task_execution_routing",
    )
}

fn resolve_frontier_proposal_token_index(
    instance: &BpmnInstanceState,
    proposal: &BpmnFrontierExecutionProposal,
) -> Option<usize> {
    let token_index = token_index_for_id(instance, proposal.token_id)?;
    let token = instance.active_tokens.get(token_index)?;
    (token.node_index == proposal.node_index
        && token.incoming_edge_index == proposal.incoming_edge_index)
        .then_some(token_index)
}

fn token_index_for_id(instance: &BpmnInstanceState, token_id: u64) -> Option<usize> {
    instance
        .active_tokens
        .iter()
        .position(|token| token.token_id == token_id)
}

fn token_index_for_node(instance: &BpmnInstanceState, node_index: BpmnNodeIndex) -> Option<usize> {
    instance
        .active_tokens
        .iter()
        .position(|token| token.node_index == node_index)
}

fn clear_pending_host_work(instance: &mut BpmnInstanceState, token_id: u64) {
    instance
        .pending_host_work
        .retain(|pending| pending.token_id != token_id);
}

fn clear_boundary_wait_for_node(instance: &mut BpmnInstanceState, node_index: BpmnNodeIndex) {
    instance
        .waits
        .retain(|wait| wait.blocking_node_index != Some(node_index));
}

fn next_token_id(instance: &BpmnInstanceState) -> u64 {
    instance
        .active_tokens
        .iter()
        .map(|token| token.token_id)
        .max()
        .unwrap_or(instance.sequence)
        .max(instance.sequence)
        + 1
}

fn record_join_arrival(
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

fn consume_join_activation(
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

pub(super) fn merge_output_data(
    variables: &mut serde_json::Value,
    output_data: &serde_json::Value,
) {
    if let Some(obj) = output_data.as_object() {
        for (key, value) in obj {
            variables[key] = value.clone();
        }
    }
}

fn node_matches_pending_kind(node_kind: &BpmnNodeKind, pending_kind: &PendingHostWorkKind) -> bool {
    matches!(
        (node_kind, pending_kind),
        (BpmnNodeKind::ServiceTask, PendingHostWorkKind::Service)
            | (BpmnNodeKind::UserTask, PendingHostWorkKind::User)
            | (BpmnNodeKind::ManualTask, PendingHostWorkKind::Manual)
            | (
                BpmnNodeKind::BusinessRuleTask,
                PendingHostWorkKind::BusinessRule
            )
    )
}

fn pending_host_kind_name(kind: &PendingHostWorkKind) -> &'static str {
    match kind {
        PendingHostWorkKind::Service => "service",
        PendingHostWorkKind::User => "user",
        PendingHostWorkKind::Manual => "manual",
        PendingHostWorkKind::BusinessRule => "business_rule",
    }
}

fn standard_loop_should_continue(
    process: &BpmnProcessSpec,
    node_index: BpmnNodeIndex,
    loop_spec: &BpmnStandardLoopSpec,
    completed_iterations: u32,
    variables: &serde_json::Value,
) -> Result<bool> {
    if let Some(loop_maximum) = loop_spec.loop_maximum
        && completed_iterations >= loop_maximum
    {
        return Ok(false);
    }

    let Some(loop_condition) = loop_spec.loop_condition.as_deref() else {
        return Ok(true);
    };
    evaluate_standard_loop_condition(process, node_index, loop_condition, variables)
}

fn evaluate_standard_loop_condition(
    process: &BpmnProcessSpec,
    node_index: BpmnNodeIndex,
    loop_condition: &str,
    variables: &serde_json::Value,
) -> Result<bool> {
    let trimmed = loop_condition.trim();
    let (negated, path) = match trimmed.strip_prefix("not ") {
        Some(path) => (true, path.trim()),
        None => (false, trimmed),
    };
    let value = resolve_boolean_variable_path(variables, path).ok_or_else(|| {
        BpmnEngineError::UnsupportedLoopConfiguration {
            process_id: process.key.process_id.to_string(),
            node_id: process.nodes[node_index as usize].bpmn_id.to_string(),
            detail: "loop_condition_variable_unresolved",
        }
    })?;
    Ok(if negated { !value } else { value })
}

fn resolve_boolean_variable_path(variables: &serde_json::Value, path: &str) -> Option<bool> {
    let mut current = variables;
    for segment in path.split('.') {
        current = current.get(segment)?;
    }
    current.as_bool()
}

fn build_wait_registration(
    process: &BpmnProcessSpec,
    node_index: BpmnNodeIndex,
    blocking_node_index: Option<BpmnNodeIndex>,
) -> Result<WaitRegistration> {
    let node = &process.nodes[node_index as usize];
    let event = process.event_for_node(node_index).ok_or_else(|| {
        BpmnEngineError::MissingRequiredNodeElement {
            process_id: process.key.process_id.to_string(),
            node_id: node.bpmn_id.to_string(),
            element: "event_definition",
        }
    })?;
    let wait_kind = match event.kind {
        BpmnEventKind::Message | BpmnEventKind::Signal => WaitKind::ExternalEvent,
        BpmnEventKind::Timer => WaitKind::Timer,
        BpmnEventKind::Conditional => {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "advance_instance_conditional_event_wait",
            });
        }
    };

    Ok(WaitRegistration {
        node_index,
        blocking_node_index,
        kind: wait_kind,
        event_kind: Some(event.kind.clone()),
        event_reference: event.reference_id.as_ref().map(ToString::to_string),
        event_name: event.name.as_ref().map(ToString::to_string),
        timer: event.timer.clone(),
        correlation_key: event.reference_id.as_ref().map(ToString::to_string),
    })
}

fn block_on_host_work(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    kind: PendingHostWorkKind,
    now_ms: u64,
) -> Result<()> {
    set_node_status(instance, node_index, NodeRuntimeStatus::Executing);
    let token_id = instance
        .active_tokens
        .get(current_token_index)
        .map(|token| token.token_id)
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "block_on_host_work_missing_active_token",
        })?;
    let pending = PendingHostWork {
        token_id,
        node_index,
        kind,
        decision: None,
        work_id: None,
    };
    push_pending_host_work(instance, pending);
    arm_boundary_timer_wait(process, instance, node_index)?;
    instance.suspend_reason = None;
    record_transition(instance, now_ms, InstanceLifecycle::Waiting);
    Ok(())
}

fn block_on_business_rule_work(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    decision: crate::dmn::DmnDecisionRef,
    now_ms: u64,
) -> Result<()> {
    set_node_status(instance, node_index, NodeRuntimeStatus::Executing);
    let token_id = instance
        .active_tokens
        .get(current_token_index)
        .map(|token| token.token_id)
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "block_on_business_rule_work_missing_active_token",
        })?;
    let pending = PendingHostWork {
        token_id,
        node_index,
        kind: PendingHostWorkKind::BusinessRule,
        decision: Some(decision),
        work_id: None,
    };
    push_pending_host_work(instance, pending);
    arm_boundary_timer_wait(process, instance, node_index)?;
    instance.suspend_reason = None;
    record_transition(instance, now_ms, InstanceLifecycle::Waiting);
    Ok(())
}

fn arm_boundary_timer_wait(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> Result<()> {
    clear_boundary_wait_for_node(instance, node_index);

    let Some(boundary) = process.boundary_event_for_attached_node(node_index) else {
        return Ok(());
    };

    if !boundary.cancel_activity {
        return Err(BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: process.key.process_id.to_string(),
            node_id: boundary.bpmn_id.to_string(),
            detail: "non_interrupting_boundary_event",
        });
    }

    let event = process.event_for_node(boundary.index).ok_or_else(|| {
        BpmnEngineError::MissingRequiredNodeElement {
            process_id: process.key.process_id.to_string(),
            node_id: boundary.bpmn_id.to_string(),
            element: "event_definition",
        }
    })?;
    if event.kind != BpmnEventKind::Timer {
        return Err(BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: process.key.process_id.to_string(),
            node_id: boundary.bpmn_id.to_string(),
            detail: "unsupported_boundary_event_kind",
        });
    }

    instance.waits.push(build_wait_registration(
        process,
        boundary.index,
        Some(node_index),
    )?);
    Ok(())
}

fn push_pending_host_work(instance: &mut BpmnInstanceState, pending: PendingHostWork) {
    clear_pending_host_work(instance, pending.token_id);
    instance.pending_host_work.push(pending);
    instance
        .pending_host_work
        .sort_by_key(|pending| (pending.token_id, pending.node_index));
}
