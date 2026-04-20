use super::scope::*;
use super::state;

pub(super) fn build_wait_registration(
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
        BpmnEventKind::Error => {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "advance_instance_error_event_wait",
            });
        }
        BpmnEventKind::Cancel => {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "advance_instance_cancel_event_wait",
            });
        }
        BpmnEventKind::Compensation => {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "advance_instance_compensation_event_wait",
            });
        }
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

pub(super) fn block_on_host_work(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    kind: PendingHostWorkKind,
    now_ms: u64,
) -> Result<()> {
    state::set_node_status(instance, node_index, NodeRuntimeStatus::Executing);
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
    state::record_transition(instance, now_ms, InstanceLifecycle::Waiting);
    Ok(())
}

pub(super) fn block_on_business_rule_work(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    node_index: BpmnNodeIndex,
    decision: crate::dmn_model_api::DmnDecisionRef,
    now_ms: u64,
) -> Result<()> {
    state::set_node_status(instance, node_index, NodeRuntimeStatus::Executing);
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
    state::record_transition(instance, now_ms, InstanceLifecycle::Waiting);
    Ok(())
}

fn arm_boundary_timer_wait(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> Result<()> {
    state::clear_boundary_wait_for_node(instance, node_index);

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
    state::clear_pending_host_work(instance, pending.token_id);
    instance.pending_host_work.push(pending);
    instance
        .pending_host_work
        .sort_by_key(|pending| (pending.token_id, pending.node_index));
}
