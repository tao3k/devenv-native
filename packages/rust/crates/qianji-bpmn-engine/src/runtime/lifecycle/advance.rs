use super::scope::{
    Borrow, BpmnAdvanceOutcome, BpmnEngineError, BpmnEventKind, BpmnInstanceState, BpmnNodeIndex,
    BpmnNodeKind, BpmnPackage, BpmnProcessSpec, InstanceLifecycle, NodeRuntimeStatus,
    PendingHostWork, PendingHostWorkKind, PendingHostWorkResult, Result,
    evaluate_dmn_package_binding_sync,
};
use super::{
    blocking, call_activity, completion, error, escalation, event_subprocess, gateway, prepare,
    repeat, state, terminate, transaction,
};
use crate::runtime_instance_api::BpmnHumanTaskLifecycleEventKind;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn advance_active_node(
    package: &BpmnPackage,
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<Option<BpmnAdvanceOutcome>> {
    let current_node = &process.nodes[current_node_index as usize];
    if let Some(host_work_kind) = host_work_kind_for_node(&current_node.kind) {
        return advance_host_task_node(
            process,
            instance,
            current_token_index,
            current_node_index,
            host_work_kind,
            now_ms,
        );
    }

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
        BpmnNodeKind::IntermediateThrowEvent => {
            advance_intermediate_throw_event(
                package,
                process,
                instance,
                current_token_index,
                current_node_index,
                now_ms,
            )?;
            Ok(None)
        }
        BpmnNodeKind::IntermediateCatchEvent => {
            advance_intermediate_catch_event(
                process,
                instance,
                current_token_index,
                current_node_index,
                now_ms,
            )?;
            Ok(None)
        }
        BpmnNodeKind::ReceiveTask => {
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
        BpmnNodeKind::SendTask
        | BpmnNodeKind::ServiceTask
        | BpmnNodeKind::ScriptTask
        | BpmnNodeKind::UserTask
        | BpmnNodeKind::ManualTask => {
            unreachable!("host-blocking task kinds return before node dispatch")
        }
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

fn host_work_kind_for_node(node_kind: &BpmnNodeKind) -> Option<PendingHostWorkKind> {
    match node_kind {
        BpmnNodeKind::SendTask => Some(PendingHostWorkKind::Send),
        BpmnNodeKind::ServiceTask => Some(PendingHostWorkKind::Service),
        BpmnNodeKind::ScriptTask => Some(PendingHostWorkKind::Script),
        BpmnNodeKind::UserTask => Some(PendingHostWorkKind::User),
        BpmnNodeKind::ManualTask => Some(PendingHostWorkKind::Manual),
        _ => None,
    }
}

fn advance_start_event(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<()> {
    if start_event_should_wait(process, instance, current_node_index)? {
        return call_activity::register_intermediate_wait(
            process,
            instance,
            current_node_index,
            now_ms,
        );
    }
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

fn start_event_should_wait(
    process: &BpmnProcessSpec,
    instance: &BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> Result<bool> {
    let Some(event) = process.event_for_node(node_index) else {
        return Ok(false);
    };
    match event.kind {
        BpmnEventKind::Message | BpmnEventKind::Signal | BpmnEventKind::Timer => Ok(true),
        BpmnEventKind::Conditional => {
            super::conditional_event_is_satisfied(process, node_index, &instance.variables)
                .map(|ready| !ready)
        }
        BpmnEventKind::Cancel
        | BpmnEventKind::Compensation
        | BpmnEventKind::Error
        | BpmnEventKind::Escalation
        | BpmnEventKind::Terminate => Err(BpmnEngineError::UnsupportedEventConfiguration {
            process_id: process.key.process_id.to_string(),
            node_id: process.nodes[node_index as usize].bpmn_id.to_string(),
            detail: "unsupported_start_event_definition",
        }),
    }
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
                if instance.call_stack.is_empty() {
                    return Ok(Some(error::fail_root_process(
                        process,
                        instance,
                        current_token_index,
                        current_node_index,
                        event.reference_id.as_deref(),
                        now_ms,
                    )?));
                }
                error::error_subprocess_shell(
                    package,
                    instance,
                    current_token_index,
                    current_node_index,
                    event.reference_id.as_deref(),
                    now_ms,
                )?;
                return Ok(None);
            }
            BpmnEventKind::Escalation => {
                if instance.call_stack.is_empty() {
                    return Err(BpmnEngineError::UnsupportedEventConfiguration {
                        process_id: process.key.process_id.to_string(),
                        node_id: process.nodes[current_node_index as usize]
                            .bpmn_id
                            .to_string(),
                        detail: "escalation_end_requires_supported_parent_boundary",
                    });
                }
                escalation::escalation_subprocess_shell(
                    package,
                    instance,
                    current_token_index,
                    current_node_index,
                    event.reference_id.as_deref(),
                    now_ms,
                )?;
                return Ok(None);
            }
            BpmnEventKind::Compensation => {
                if event.wait_for_completion {
                    transaction::throw_compensation_end_event(
                        package,
                        process,
                        instance,
                        current_token_index,
                        current_node_index,
                        event.reference_id.as_deref(),
                        now_ms,
                    )?;
                } else {
                    transaction::throw_compensation_end_event_async(
                        package,
                        process,
                        instance,
                        current_token_index,
                        current_node_index,
                        event.reference_id.as_deref(),
                        now_ms,
                    )?;
                }
                return Ok(None);
            }
            BpmnEventKind::Terminate => {
                return terminate::terminate_end_event(
                    package,
                    instance,
                    current_token_index,
                    current_node_index,
                    now_ms,
                );
            }
            _ => {}
        }
    }

    advance_plain_end_event(
        package,
        instance,
        current_token_index,
        process,
        current_node_index,
        now_ms,
    )
}

fn advance_plain_end_event(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    process: &BpmnProcessSpec,
    current_node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<Option<BpmnAdvanceOutcome>> {
    state::set_node_status(instance, current_node_index, NodeRuntimeStatus::Completed);
    let _ = state::remove_active_token(instance, current_token_index);
    if !instance.active_tokens.is_empty() {
        state::record_transition(instance, now_ms, InstanceLifecycle::Running);
        return Ok(None);
    }
    event_subprocess::clear_event_subprocess_waits(process, instance);
    if instance.call_stack.is_empty() {
        if !instance.pending_host_work.is_empty() {
            instance.suspend_reason = None;
            state::record_transition(instance, now_ms, InstanceLifecycle::Waiting);
            return Ok(Some(BpmnAdvanceOutcome::BlockedOnHost(
                instance.pending_host_work.clone(),
            )));
        }
        if !instance.waits.is_empty() {
            instance.suspend_reason = None;
            state::record_transition(instance, now_ms, InstanceLifecycle::Waiting);
            return Ok(Some(BpmnAdvanceOutcome::WaitingExternalEvent));
        }
        instance.suspend_reason = None;
        state::record_transition(instance, now_ms, InstanceLifecycle::Completed);
        return Ok(Some(BpmnAdvanceOutcome::Completed));
    }

    call_activity::complete_call_activity(package, instance, now_ms)?;
    Ok(None)
}

fn advance_intermediate_catch_event(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<()> {
    if conditional_catch_event_is_ready(process, instance, current_node_index)? {
        return completion::complete_node_and_route(
            process,
            instance,
            current_token_index,
            current_node_index,
            now_ms,
            "advance_instance_conditional_catch_routing",
        );
    }
    call_activity::register_intermediate_wait(process, instance, current_node_index, now_ms)
}

fn conditional_catch_event_is_ready(
    process: &BpmnProcessSpec,
    instance: &BpmnInstanceState,
    node_index: BpmnNodeIndex,
) -> Result<bool> {
    let Some(event) = process.event_for_node(node_index) else {
        return Ok(false);
    };
    if event.kind != BpmnEventKind::Conditional {
        return Ok(false);
    }
    super::conditional_event_is_satisfied(process, node_index, &instance.variables)
}

fn advance_intermediate_throw_event(
    package: &BpmnPackage,
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    current_token_index: usize,
    current_node_index: BpmnNodeIndex,
    now_ms: u64,
) -> Result<()> {
    let event = process.event_for_node(current_node_index).ok_or_else(|| {
        BpmnEngineError::MissingRequiredNodeElement {
            process_id: process.key.process_id.to_string(),
            node_id: process.nodes[current_node_index as usize]
                .bpmn_id
                .to_string(),
            element: "event_definition",
        }
    })?;
    match event.kind {
        BpmnEventKind::Compensation => {
            if event.wait_for_completion {
                transaction::throw_compensation_intermediate_event(
                    process,
                    instance,
                    current_token_index,
                    current_node_index,
                    event.reference_id.as_deref(),
                    now_ms,
                )
            } else {
                transaction::throw_compensation_intermediate_event_async(
                    process,
                    instance,
                    current_token_index,
                    current_node_index,
                    event.reference_id.as_deref(),
                    now_ms,
                )
            }
        }
        BpmnEventKind::Escalation => {
            if instance.call_stack.is_empty() {
                return Err(BpmnEngineError::UnsupportedEventConfiguration {
                    process_id: process.key.process_id.to_string(),
                    node_id: process.nodes[current_node_index as usize]
                        .bpmn_id
                        .to_string(),
                    detail: "escalation_throw_requires_supported_parent_boundary",
                });
            }
            escalation::escalation_subprocess_shell(
                package,
                instance,
                current_token_index,
                current_node_index,
                event.reference_id.as_deref(),
                now_ms,
            )
        }
        _ => Err(BpmnEngineError::UnsupportedOperation {
            operation: "advance_instance_intermediate_throw_event_kind",
        }),
    }
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
    if let Some(evaluation) = evaluate_dmn_package_binding_sync(package, &decision, &variables)? {
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
    let pending_process_id = pending
        .process_id
        .as_deref()
        .unwrap_or(instance.process.process_id.as_ref())
        .to_string();
    let process = resolve_process_for_pending_host_work(package, pending_process_id.as_str())?;

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
    let mapped_output = map_task_completion_output(
        &pending,
        pending_process_id.as_str(),
        current_node.bpmn_id.as_ref(),
        result.data(),
    )?;

    state::clear_pending_host_work(instance, token_id);
    let token_index = state::token_index_for_id(instance, token_id);
    if token_index.is_none()
        && current_node.is_for_compensation
        && transaction::detached_compensation_matches_pending(instance, &pending)
    {
        transaction::complete_detached_compensation_handler(package, instance, completed_at_ms)?;
        record_human_task_completion_event(instance, &pending, completed_at_ms);
        maybe_clear_boundary_wait_after_host_completion(
            instance,
            pending_process_id.as_str(),
            pending.node_index,
        );
        return Ok(BpmnAdvanceOutcome::Advanced);
    }
    let token_index = token_index.ok_or(BpmnEngineError::UnsupportedOperation {
        operation: "apply_pending_host_work_result_missing_active_token",
    })?;
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
        record_human_task_completion_event(instance, &pending, completed_at_ms);
        maybe_clear_boundary_wait_after_host_completion(
            instance,
            pending_process_id.as_str(),
            pending.node_index,
        );
        return Ok(BpmnAdvanceOutcome::Advanced);
    }
    completion::complete_local_task_execution_with_variable_output(
        process,
        instance,
        token_index,
        pending.node_index,
        result.data(),
        &mapped_output,
        completed_at_ms,
    )?;
    record_human_task_completion_event(instance, &pending, completed_at_ms);
    maybe_clear_boundary_wait_after_host_completion(
        instance,
        pending_process_id.as_str(),
        pending.node_index,
    );

    Ok(BpmnAdvanceOutcome::Advanced)
}

fn map_task_completion_output(
    pending: &PendingHostWork,
    process_id: &str,
    activity_id: &str,
    data: &Value,
) -> Result<Value> {
    let Some(task_io) = pending
        .task_io
        .as_ref()
        .filter(|task_io| !task_io.outputs.is_empty())
    else {
        return Err(BpmnEngineError::MissingTaskOutputMapping {
            process_id: process_id.to_string(),
            activity_id: activity_id.to_string(),
        });
    };
    let Some(data) = data.as_object() else {
        return Err(BpmnEngineError::TaskCompletionDataNotObject {
            process_id: process_id.to_string(),
            activity_id: activity_id.to_string(),
        });
    };
    let mut declared_targets = BTreeMap::<String, (String, bool)>::new();
    for output in &task_io.outputs {
        declared_targets.insert(
            output.name.to_string(),
            (output.target_ref.to_string(), output.required),
        );
    }
    let declared_fields = declared_targets.keys().cloned().collect::<BTreeSet<_>>();
    for (field, (_, required)) in &declared_targets {
        if *required && !data.contains_key(field.as_str()) {
            return Err(BpmnEngineError::MissingTaskCompletionField {
                process_id: process_id.to_string(),
                activity_id: activity_id.to_string(),
                field: field.clone(),
            });
        }
    }
    for field in data.keys() {
        if !declared_fields.contains(field) {
            return Err(BpmnEngineError::UndeclaredTaskCompletionField {
                process_id: process_id.to_string(),
                activity_id: activity_id.to_string(),
                field: field.clone(),
            });
        }
    }
    let mut mapped = Value::Object(Map::new());
    for (field, (target_ref, _)) in declared_targets {
        let Some(value) = data.get(field.as_str()).cloned() else {
            continue;
        };
        assign_value_path(&mut mapped, target_ref.as_str(), value)?;
    }
    Ok(mapped)
}

fn assign_value_path(variables: &mut Value, path: &str, value: Value) -> Result<()> {
    let Some(object) = variables.as_object_mut() else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "task_output_assignment_non_object_root",
        });
    };
    let mut segments = path
        .split('.')
        .filter(|segment| !segment.is_empty())
        .peekable();
    let Some(last_segment) = segments.next_back() else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "task_output_assignment_empty_target",
        });
    };

    let mut current = object;
    for segment in segments {
        let entry = current
            .entry(segment.to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        let Some(next) = entry.as_object_mut() else {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "task_output_assignment_conflicting_non_object_segment",
            });
        };
        current = next;
    }
    current.insert(last_segment.to_string(), value);
    Ok(())
}

fn record_human_task_completion_event(
    instance: &mut BpmnInstanceState,
    pending: &PendingHostWork,
    completed_at_ms: u64,
) {
    state::record_human_task_lifecycle_event(
        instance,
        BpmnHumanTaskLifecycleEventKind::Completed,
        pending,
        completed_at_ms,
        pending.claim.as_ref().map(|claim| claim.claimant.clone()),
    );
}

fn maybe_clear_boundary_wait_after_host_completion(
    instance: &mut BpmnInstanceState,
    process_id: &str,
    node_index: BpmnNodeIndex,
) {
    if process_id == instance.process.process_id.as_ref()
        && !state::has_pending_host_work_for_process_node(instance, process_id, node_index)
        && state::token_index_for_node(instance, node_index).is_none()
    {
        state::clear_boundary_wait_for_node(instance, node_index);
    }
    if instance.pending_host_work.is_empty() && instance.waits.is_empty() {
        instance.suspend_reason = None;
    }
}

fn resolve_process_for_pending_host_work<'a>(
    package: &'a BpmnPackage,
    process_id: &str,
) -> Result<&'a BpmnProcessSpec> {
    package
        .find_process(process_id)
        .ok_or_else(|| BpmnEngineError::MissingProcess {
            process_id: process_id.to_string(),
        })
}
