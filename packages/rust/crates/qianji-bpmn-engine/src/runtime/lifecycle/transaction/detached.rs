use crate::runtime::lifecycle::scope::{
    BpmnEngineError, BpmnInstanceState, BpmnNodeIndex, BpmnNodeKind, BpmnPackage, BpmnProcessSpec,
    InstanceLifecycle, PendingHostWork, PendingHostWorkKind, Result,
    evaluate_dmn_package_binding_sync,
};
use crate::runtime::lifecycle::state;
use crate::runtime_instance_api::BpmnHumanTaskLifecycleEventKind;
use crate::runtime_instance_api::DetachedTransactionCompensationState;

enum DetachedCompensationProgress {
    Continue,
    Blocked,
}

struct DetachedHostWorkSpec {
    kind: PendingHostWorkKind,
    decision: Option<crate::dmn_model_api::DmnDecisionRef>,
    script_format: Option<String>,
    script_body: Option<String>,
    task_io: Option<crate::ir_node_api::BpmnTaskIoSpec>,
}

pub(super) fn install_detached_compensation_queue(
    instance: &mut BpmnInstanceState,
    process: &BpmnProcessSpec,
    pending_handler_node_indices: Vec<BpmnNodeIndex>,
) -> Result<()> {
    if instance.detached_transaction_compensation.is_some() {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "throw_compensation_end_event_detached_queue_already_active",
        });
    }

    instance.detached_transaction_compensation = Some(DetachedTransactionCompensationState {
        process: process.key.clone(),
        process_index: instance.process_index,
        pending_handler_node_indices: pending_handler_node_indices.into_iter().rev().collect(),
    });
    Ok(())
}

pub(super) fn detached_compensation_matches_pending(
    instance: &BpmnInstanceState,
    pending: &PendingHostWork,
) -> bool {
    instance
        .detached_transaction_compensation
        .as_ref()
        .is_some_and(|state| {
            pending.process_id.as_deref() == Some(state.process.process_id.as_ref())
        })
}

pub(super) fn continue_detached_compensation_queue(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    now_ms: u64,
) -> Result<()> {
    loop {
        let next_handler = next_detached_handler(instance);
        let Some((process_key, process_index, handler_node_index)) = next_handler else {
            instance.detached_transaction_compensation = None;
            record_after_detached_progress(instance, now_ms);
            return Ok(());
        };
        let process = resolve_detached_process(package, &process_key, process_index)?;
        match advance_detached_compensation_handler(
            package,
            instance,
            &process_key,
            process,
            handler_node_index,
            now_ms,
        )? {
            DetachedCompensationProgress::Continue => {}
            DetachedCompensationProgress::Blocked => return Ok(()),
        }
    }
}

fn next_detached_handler(
    instance: &mut BpmnInstanceState,
) -> Option<(crate::ir::ProcessKey, u32, BpmnNodeIndex)> {
    instance
        .detached_transaction_compensation
        .as_mut()
        .and_then(|state| {
            state
                .pending_handler_node_indices
                .pop()
                .map(|node_index| (state.process.clone(), state.process_index, node_index))
        })
}

fn advance_detached_compensation_handler(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    process_key: &crate::ir::ProcessKey,
    process: &BpmnProcessSpec,
    handler_node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<DetachedCompensationProgress> {
    let node = detached_handler_node(process, handler_node_index)?;
    if let Some(spec) = detached_host_work_spec(node) {
        enqueue_detached_host_work(
            instance,
            process_key.process_id.as_ref(),
            handler_node_index,
            node.bpmn_id.as_ref(),
            spec,
            now_ms,
        );
        return Ok(DetachedCompensationProgress::Blocked);
    }
    match node.kind {
        BpmnNodeKind::BusinessRuleTask => advance_detached_business_rule_handler(
            package,
            instance,
            process_key,
            node,
            handler_node_index,
            now_ms,
        ),
        _ => Err(BpmnEngineError::UnsupportedOperation {
            operation: "continue_detached_compensation_handler_unsupported_kind",
        }),
    }
}

fn detached_handler_node(
    process: &BpmnProcessSpec,
    handler_node_index: BpmnNodeIndex,
) -> Result<&crate::ir_node_api::BpmnNodeSpec> {
    process
        .nodes
        .get(handler_node_index as usize)
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "continue_detached_compensation_handler_missing_node",
        })
}

fn detached_host_work_spec(
    node: &crate::ir_node_api::BpmnNodeSpec,
) -> Option<DetachedHostWorkSpec> {
    match node.kind {
        BpmnNodeKind::ServiceTask => Some(DetachedHostWorkSpec {
            kind: PendingHostWorkKind::Service,
            decision: None,
            script_format: None,
            script_body: None,
            task_io: node.task_io.clone(),
        }),
        BpmnNodeKind::ScriptTask => Some(DetachedHostWorkSpec {
            kind: PendingHostWorkKind::Script,
            decision: None,
            script_format: node
                .script_task
                .as_ref()
                .and_then(|script| script.script_format.as_ref())
                .map(ToString::to_string),
            script_body: node
                .script_task
                .as_ref()
                .and_then(|script| script.script_body.as_ref())
                .map(ToString::to_string),
            task_io: node.task_io.clone(),
        }),
        BpmnNodeKind::UserTask => Some(DetachedHostWorkSpec {
            kind: PendingHostWorkKind::User,
            decision: None,
            script_format: None,
            script_body: None,
            task_io: node.task_io.clone(),
        }),
        BpmnNodeKind::ManualTask => Some(DetachedHostWorkSpec {
            kind: PendingHostWorkKind::Manual,
            decision: None,
            script_format: None,
            script_body: None,
            task_io: node.task_io.clone(),
        }),
        _ => None,
    }
}

fn advance_detached_business_rule_handler(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    process_key: &crate::ir::ProcessKey,
    node: &crate::ir_node_api::BpmnNodeSpec,
    handler_node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<DetachedCompensationProgress> {
    let decision =
        node.decision
            .clone()
            .ok_or_else(|| BpmnEngineError::MissingBusinessRuleDecisionRef {
                process_id: process_key.process_id.to_string(),
                node_id: node.bpmn_id.to_string(),
            })?;
    if evaluate_dmn_package_binding_sync(package, &decision, &instance.variables)?.is_some() {
        return Ok(DetachedCompensationProgress::Continue);
    }
    enqueue_detached_host_work(
        instance,
        process_key.process_id.as_ref(),
        handler_node_index,
        node.bpmn_id.as_ref(),
        DetachedHostWorkSpec {
            kind: PendingHostWorkKind::BusinessRule,
            decision: Some(decision),
            script_format: None,
            script_body: None,
            task_io: node.task_io.clone(),
        },
        now_ms,
    );
    Ok(DetachedCompensationProgress::Blocked)
}

fn resolve_detached_process<'a>(
    package: &'a BpmnPackage,
    process_key: &crate::ir::ProcessKey,
    process_index: u32,
) -> Result<&'a BpmnProcessSpec> {
    package
        .processes
        .get(process_index as usize)
        .filter(|process| process.key.process_id == process_key.process_id)
        .or_else(|| package.find_process(process_key.process_id.as_ref()))
        .ok_or_else(|| BpmnEngineError::MissingProcess {
            process_id: process_key.process_id.to_string(),
        })
}

fn enqueue_detached_host_work(
    instance: &mut BpmnInstanceState,
    process_id: &str,
    node_index: BpmnNodeIndex,
    activity_id: &str,
    spec: DetachedHostWorkSpec,
    now_ms: u64,
) {
    let token_id = state::allocate_token_id(instance);
    let pending = PendingHostWork {
        token_id,
        process_id: Some(process_id.to_string()),
        node_index,
        activity_id: Some(activity_id.to_string()),
        kind: spec.kind,
        decision: spec.decision,
        lane: None,
        script_format: spec.script_format,
        script_body: spec.script_body,
        human_task_form: None,
        human_task_assignment: None,
        task_io: spec.task_io,
        claim: None,
        event_reference: None,
        event_name: None,
        work_id: None,
    };
    instance.pending_host_work.push(pending.clone());
    instance
        .pending_host_work
        .sort_by_key(|pending| (pending.token_id, pending.node_index));
    state::record_human_task_lifecycle_event(
        instance,
        BpmnHumanTaskLifecycleEventKind::Created,
        &pending,
        now_ms,
        None,
    );
    instance.suspend_reason = None;
    state::record_transition(
        instance,
        now_ms,
        if instance.active_tokens.is_empty() {
            InstanceLifecycle::Waiting
        } else {
            InstanceLifecycle::Running
        },
    );
}

fn record_after_detached_progress(instance: &mut BpmnInstanceState, now_ms: u64) {
    let lifecycle = if !instance.active_tokens.is_empty() {
        InstanceLifecycle::Running
    } else if !instance.pending_host_work.is_empty() || !instance.waits.is_empty() {
        InstanceLifecycle::Waiting
    } else if instance.call_stack.is_empty() {
        instance.suspend_reason = None;
        InstanceLifecycle::Completed
    } else {
        InstanceLifecycle::Running
    };
    state::record_transition(instance, now_ms, lifecycle);
}
