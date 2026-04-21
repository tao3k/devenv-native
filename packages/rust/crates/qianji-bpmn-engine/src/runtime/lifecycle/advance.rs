use super::scope::{
    Borrow, BpmnAdvanceOutcome, BpmnEngineError, BpmnEventKind, BpmnInstanceState, BpmnNodeIndex,
    BpmnNodeKind, BpmnPackage, BpmnProcessSpec, DmnEvaluationRequest, InstanceLifecycle,
    NodeRuntimeStatus, PendingHostWorkKind, PendingHostWorkResult, Result,
    evaluate_dmn_decision_sync, resolve_process_for_instance,
};
use super::{blocking, call_activity, completion, gateway, prepare, repeat, state, transaction};

pub(super) fn advance_active_node(
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
            call_activity::register_intermediate_wait(
                process,
                instance,
                current_node_index,
                now_ms,
            )?;
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
        BpmnNodeKind::Gateway => gateway::advance_gateway(
            process,
            instance,
            current_token_index,
            current_node_index,
            now_ms,
        ),
        BpmnNodeKind::SubProcess => {
            call_activity::enter_call_activity(
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
    let edge_index = state::resolve_single_outgoing_edge(
        process,
        current_node_index,
        "advance_instance_start_event_routing",
    )?;
    let next_node_index = process.edges[edge_index as usize].to;
    state::set_node_status(instance, current_node_index, NodeRuntimeStatus::Completed);
    state::set_active_node_index(instance, current_token_index, edge_index, next_node_index);
    state::set_node_status(instance, next_node_index, NodeRuntimeStatus::Queued);
    state::record_transition(instance, now_ms, InstanceLifecycle::Running);
    Ok(())
}

fn advance_end_event(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    process: &BpmnProcessSpec,
    current_node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<Option<BpmnAdvanceOutcome>> {
    if let Some(event) = process.event_for_node(current_node_index) {
        match event.kind {
            BpmnEventKind::Cancel => {
                transaction::cancel_transaction_shell(
                    package,
                    instance,
                    current_token_index,
                    current_node_index,
                    now_ms,
                )?;
                return Ok(None);
            }
            BpmnEventKind::Error => {
                transaction::error_transaction_shell(
                    package,
                    instance,
                    current_token_index,
                    current_node_index,
                    event.reference_id.as_deref(),
                    now_ms,
                )?;
                return Ok(None);
            }
            _ => {}
        }
    }

    state::set_node_status(instance, current_node_index, NodeRuntimeStatus::Completed);
    let _ = state::remove_active_token(instance, current_token_index);
    if !instance.active_tokens.is_empty() {
        state::record_transition(instance, now_ms, InstanceLifecycle::Running);
        return Ok(None);
    }
    if instance.call_stack.is_empty() {
        instance.pending_host_work.clear();
        instance.waits.clear();
        instance.suspend_reason = None;
        state::record_transition(instance, now_ms, InstanceLifecycle::Completed);
        return Ok(Some(BpmnAdvanceOutcome::Completed));
    }

    call_activity::complete_call_activity(package, instance, now_ms)?;
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
    if prepare::prepare_standard_loop_iteration(
        process,
        instance,
        current_token_index,
        current_node_index,
        now_ms,
    )? || prepare::prepare_sequential_multi_instance_iteration(
        process,
        instance,
        current_token_index,
        current_node_index,
        now_ms,
    )? || prepare::prepare_parallel_multi_instance_iteration(
        process,
        instance,
        current_token_index,
        current_node_index,
        process.nodes[current_node_index as usize].repeat.as_ref(),
        now_ms,
    )? {
        return Ok(None);
    }
    blocking::block_on_host_work(
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
    current_node: &crate::ir_node_api::BpmnNodeSpec,
    current_node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<Option<BpmnAdvanceOutcome>> {
    if prepare::prepare_standard_loop_iteration(
        process,
        instance,
        current_token_index,
        current_node_index,
        now_ms,
    )? || prepare::prepare_sequential_multi_instance_iteration(
        process,
        instance,
        current_token_index,
        current_node_index,
        now_ms,
    )? || prepare::prepare_parallel_multi_instance_iteration(
        process,
        instance,
        current_token_index,
        current_node_index,
        process.nodes[current_node_index as usize].repeat.as_ref(),
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
    let token_id = instance
        .active_tokens
        .get(current_token_index)
        .map(|token| token.token_id)
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "advance_instance_business_rule_missing_active_token",
        })?;
    let variables =
        repeat::materialize_node_execution_variables(instance, current_node_index, token_id)?;
    if let Some(definition) = package.find_dmn_decision(&decision)? {
        let evaluation = evaluate_dmn_decision_sync(
            definition,
            &DmnEvaluationRequest::new(decision.clone(), variables),
        )?;
        if current_node.is_for_compensation
            && transaction::transaction_compensation_is_running(instance)
        {
            transaction::complete_compensation_handler(
                package,
                process,
                instance,
                current_token_index,
                current_node_index,
                now_ms,
            )?;
            return Ok(None);
        }
        completion::complete_local_task_execution(
            process,
            instance,
            current_token_index,
            current_node_index,
            &evaluation.output,
            now_ms,
        )?;
        return Ok(None);
    }
    blocking::block_on_business_rule_work(
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
pub(crate) fn apply_pending_host_work_result_impl(
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
            expected: repeat::pending_host_kind_name(&pending.kind),
            actual: result.kind_name(),
        });
    }

    let current_node = &process.nodes[pending.node_index as usize];
    if !repeat::node_matches_pending_kind(&current_node.kind, &pending.kind) {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "apply_pending_host_work_result_node_kind_mismatch",
        });
    }

    let token_index = state::token_index_for_id(instance, token_id).ok_or(
        BpmnEngineError::UnsupportedOperation {
            operation: "apply_pending_host_work_result_missing_active_token",
        },
    )?;
    state::clear_pending_host_work(instance, token_id);
    if !state::has_pending_host_work_for_node(instance, pending.node_index) {
        state::clear_boundary_wait_for_node(instance, pending.node_index);
    }
    if instance.pending_host_work.is_empty() && instance.waits.is_empty() {
        instance.suspend_reason = None;
    }
    if current_node.is_for_compensation
        && transaction::transaction_compensation_is_running(instance)
    {
        transaction::complete_compensation_handler(
            package,
            process,
            instance,
            token_index,
            pending.node_index,
            completed_at_ms,
        )?;
        return Ok(BpmnAdvanceOutcome::Advanced);
    }
    completion::complete_local_task_execution(
        process,
        instance,
        token_index,
        pending.node_index,
        result.data(),
        completed_at_ms,
    )?;

    Ok(BpmnAdvanceOutcome::Advanced)
}
