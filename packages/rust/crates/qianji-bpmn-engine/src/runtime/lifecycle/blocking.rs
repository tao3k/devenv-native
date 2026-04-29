use super::scope::{
    BpmnEngineError, BpmnEventKind, BpmnInstanceState, BpmnNodeIndex, BpmnProcessSpec,
    InstanceLifecycle, NodeRuntimeStatus, PendingHostWork, PendingHostWorkKind, Result, WaitKind,
    WaitRegistration,
};
use super::state;
use crate::runtime_instance_api::BpmnHumanTaskLifecycleEventKind;

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
        BpmnEventKind::Conditional => WaitKind::Conditional,
        BpmnEventKind::Error => {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "advance_instance_error_event_wait",
            });
        }
        BpmnEventKind::Escalation => {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "advance_instance_escalation_event_wait",
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
        BpmnEventKind::Terminate => {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "advance_instance_terminate_event_wait",
            });
        }
    };

    Ok(WaitRegistration {
        process_id: Some(process.key.process_id.to_string()),
        node_index,
        blocking_node_index,
        kind: wait_kind,
        event_kind: Some(event.kind.clone()),
        event_reference: event.reference_id.as_ref().map(ToString::to_string),
        event_name: event.name.as_ref().map(ToString::to_string),
        timer: event.timer.clone(),
        condition_expression: event.condition_expression.as_ref().map(ToString::to_string),
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
    let event_reference = send_task_event_reference(process, node_index, &kind)?;
    let event_name = send_task_event_name(process, node_index, &kind)?;
    let script_format = script_task_format(process, node_index, &kind);
    let script_body = script_task_body(process, node_index, &kind);
    let human_task_form = process
        .nodes
        .get(node_index as usize)
        .and_then(|node| node.human_task_form.clone());
    let human_task_assignment = process
        .nodes
        .get(node_index as usize)
        .and_then(|node| node.human_task_assignment.clone());
    let task_io = process
        .nodes
        .get(node_index as usize)
        .and_then(|node| node.task_io.clone());
    let lane = process
        .nodes
        .get(node_index as usize)
        .and_then(|node| node.lane.clone());
    let activity_id = process
        .nodes
        .get(node_index as usize)
        .map(|node| node.bpmn_id.to_string());
    let pending = PendingHostWork {
        token_id,
        process_id: Some(process.key.process_id.to_string()),
        node_index,
        activity_id,
        kind,
        decision: None,
        lane,
        script_format,
        script_body,
        human_task_form,
        human_task_assignment,
        task_io,
        claim: None,
        event_reference,
        event_name,
        work_id: None,
    };
    push_pending_host_work(instance, pending, now_ms);
    arm_boundary_wait(process, instance, node_index)?;
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
        process_id: Some(process.key.process_id.to_string()),
        node_index,
        activity_id: process
            .nodes
            .get(node_index as usize)
            .map(|node| node.bpmn_id.to_string()),
        kind: PendingHostWorkKind::BusinessRule,
        decision: Some(decision),
        lane: process
            .nodes
            .get(node_index as usize)
            .and_then(|node| node.lane.clone()),
        script_format: None,
        script_body: None,
        human_task_form: process
            .nodes
            .get(node_index as usize)
            .and_then(|node| node.human_task_form.clone()),
        human_task_assignment: process
            .nodes
            .get(node_index as usize)
            .and_then(|node| node.human_task_assignment.clone()),
        task_io: process
            .nodes
            .get(node_index as usize)
            .and_then(|node| node.task_io.clone()),
        claim: None,
        event_reference: None,
        event_name: None,
        work_id: None,
    };
    push_pending_host_work(instance, pending, now_ms);
    arm_boundary_wait(process, instance, node_index)?;
    instance.suspend_reason = None;
    state::record_transition(instance, now_ms, InstanceLifecycle::Waiting);
    Ok(())
}

fn arm_boundary_wait(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> Result<()> {
    state::clear_boundary_wait_for_node(instance, node_index);

    let Some(boundary) = process.boundary_event_for_attached_node(node_index) else {
        return Ok(());
    };

    let event = process.event_for_node(boundary.index).ok_or_else(|| {
        BpmnEngineError::MissingRequiredNodeElement {
            process_id: process.key.process_id.to_string(),
            node_id: boundary.bpmn_id.to_string(),
            element: "event_definition",
        }
    })?;
    if event.kind == BpmnEventKind::Compensation {
        return Ok(());
    }
    if !matches!(
        event.kind,
        BpmnEventKind::Timer | BpmnEventKind::Message | BpmnEventKind::Signal
    ) {
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

fn push_pending_host_work(instance: &mut BpmnInstanceState, pending: PendingHostWork, now_ms: u64) {
    state::clear_pending_host_work(instance, pending.token_id);
    let token_id = pending.token_id;
    let node_index = pending.node_index;
    instance.pending_host_work.push(pending);
    instance
        .pending_host_work
        .sort_by_key(|pending| (pending.token_id, pending.node_index));
    let created = instance
        .pending_host_work
        .iter()
        .find(|pending| pending.token_id == token_id && pending.node_index == node_index)
        .cloned();
    if let Some(created) = created {
        state::record_human_task_lifecycle_event(
            instance,
            BpmnHumanTaskLifecycleEventKind::Created,
            &created,
            now_ms,
            None,
        );
    }
}

fn send_task_event_reference(
    process: &BpmnProcessSpec,
    node_index: BpmnNodeIndex,
    kind: &PendingHostWorkKind,
) -> Result<Option<String>> {
    if kind != &PendingHostWorkKind::Send {
        return Ok(None);
    }
    let node = &process.nodes[node_index as usize];
    let event = process.event_for_node(node_index).ok_or_else(|| {
        BpmnEngineError::MissingRequiredNodeElement {
            process_id: process.key.process_id.to_string(),
            node_id: node.bpmn_id.to_string(),
            element: "event_definition",
        }
    })?;
    Ok(event.reference_id.as_ref().map(ToString::to_string))
}

fn send_task_event_name(
    process: &BpmnProcessSpec,
    node_index: BpmnNodeIndex,
    kind: &PendingHostWorkKind,
) -> Result<Option<String>> {
    if kind != &PendingHostWorkKind::Send {
        return Ok(None);
    }
    let node = &process.nodes[node_index as usize];
    let event = process.event_for_node(node_index).ok_or_else(|| {
        BpmnEngineError::MissingRequiredNodeElement {
            process_id: process.key.process_id.to_string(),
            node_id: node.bpmn_id.to_string(),
            element: "event_definition",
        }
    })?;
    Ok(event.name.as_ref().map(ToString::to_string))
}

fn script_task_format(
    process: &BpmnProcessSpec,
    node_index: BpmnNodeIndex,
    kind: &PendingHostWorkKind,
) -> Option<String> {
    if kind != &PendingHostWorkKind::Script {
        return None;
    }
    process.nodes[node_index as usize]
        .script_task
        .as_ref()
        .and_then(|script| script.script_format.as_ref())
        .map(ToString::to_string)
}

fn script_task_body(
    process: &BpmnProcessSpec,
    node_index: BpmnNodeIndex,
    kind: &PendingHostWorkKind,
) -> Option<String> {
    if kind != &PendingHostWorkKind::Script {
        return None;
    }
    process.nodes[node_index as usize]
        .script_task
        .as_ref()
        .and_then(|script| script.script_body.as_ref())
        .map(ToString::to_string)
}
