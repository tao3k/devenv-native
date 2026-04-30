use super::scope::{
    BpmnAdvanceOutcome, BpmnEngineError, BpmnInstanceState, BpmnNodeIndex, BpmnPackage,
    BpmnProcessSpec, BpmnSubProcessKind, InstanceLifecycle, NodeRuntimeStatus, Result,
    WaitRegistration, install_process_state, pop_call_activity_frame, resolve_process_for_instance,
    restore_call_activity_frame,
};
use super::{blocking, state};

pub(super) fn arm_event_subprocess_waits(
    package: &BpmnPackage,
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
) -> Result<()> {
    let owner_indices = process
        .nodes
        .iter()
        .filter(|node| node.subprocess_kind == Some(BpmnSubProcessKind::EventSubProcess))
        .map(|node| node.index)
        .collect::<Vec<_>>();
    for owner_index in owner_indices {
        if event_subprocess_wait_is_armed(instance, process, owner_index) {
            continue;
        }
        let _ = event_subprocess_child(package, process, owner_index)?;
        instance.waits.push(blocking::build_wait_registration(
            process,
            owner_index,
            Some(owner_index),
        )?);
    }
    Ok(())
}

pub(super) fn clear_event_subprocess_waits(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
) {
    instance.waits.retain(|wait| {
        wait.blocking_node_index.is_none_or(|blocking_node_index| {
            !is_event_subprocess_wait(process, blocking_node_index)
        })
    });
}

pub(crate) fn is_event_subprocess_wait(
    process: &BpmnProcessSpec,
    blocking_node_index: BpmnNodeIndex,
) -> bool {
    process
        .nodes
        .get(blocking_node_index as usize)
        .is_some_and(|node| node.subprocess_kind == Some(BpmnSubProcessKind::EventSubProcess))
}

pub(crate) fn apply_current_frame_event_subprocess_wait(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    wait: &WaitRegistration,
    owner_node_index: BpmnNodeIndex,
    polled_at_ms: u64,
) -> Result<BpmnAdvanceOutcome> {
    let process = resolve_process_for_instance(package, instance)?;
    ensure_wait_process(process, wait)?;
    enter_event_subprocess_body(package, process, instance, owner_node_index, polled_at_ms)
}

pub(crate) fn apply_parent_frame_event_subprocess_wait(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    wait: &WaitRegistration,
    owner_node_index: BpmnNodeIndex,
    polled_at_ms: u64,
) -> Result<BpmnAdvanceOutcome> {
    let frame = pop_call_activity_frame(instance).ok_or(BpmnEngineError::UnsupportedOperation {
        operation: "apply_event_subprocess_parent_frame_missing_frame",
    })?;
    let _ = restore_call_activity_frame(instance, frame);
    apply_current_frame_event_subprocess_wait(
        package,
        instance,
        wait,
        owner_node_index,
        polled_at_ms,
    )
}

fn event_subprocess_wait_is_armed(
    instance: &BpmnInstanceState,
    process: &BpmnProcessSpec,
    owner_node_index: BpmnNodeIndex,
) -> bool {
    instance.waits.iter().any(|wait| {
        wait.process_id.as_deref() == Some(process.key.process_id.as_ref())
            && wait.node_index == owner_node_index
            && wait.blocking_node_index == Some(owner_node_index)
    })
}

fn ensure_wait_process(process: &BpmnProcessSpec, wait: &WaitRegistration) -> Result<()> {
    if wait
        .process_id
        .as_deref()
        .is_some_and(|process_id| process_id != process.key.process_id.as_ref())
    {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "apply_event_subprocess_process_mismatch",
        });
    }
    Ok(())
}

fn enter_event_subprocess_body(
    package: &BpmnPackage,
    parent_process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    owner_node_index: BpmnNodeIndex,
    polled_at_ms: u64,
) -> Result<BpmnAdvanceOutcome> {
    let (child_process_index, child_process) =
        event_subprocess_child(package, parent_process, owner_node_index)?;

    instance.pending_host_work.clear();
    instance.waits.clear();
    instance.event_competition = None;
    instance.active_tokens.clear();
    instance.joins.clear();
    instance.standard_loops.clear();
    instance.sequential_multi_instances.clear();
    instance.parallel_multi_instances.clear();
    instance.suspend_reason = None;

    install_process_state(instance, child_process, child_process_index);
    enter_after_start_event(child_process, instance, package, polled_at_ms)?;
    Ok(BpmnAdvanceOutcome::Advanced)
}

fn enter_after_start_event(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    package: &BpmnPackage,
    now_ms: u64,
) -> Result<()> {
    let start_node_index = state::find_single_start_node(process)?;
    let edge_index = state::resolve_single_outgoing_edge(
        process,
        start_node_index,
        "enter_event_subprocess_start_routing",
    )?;
    let next_node_index = process.edges[edge_index as usize].to;
    state::set_node_status(instance, start_node_index, NodeRuntimeStatus::Completed);
    let _ = state::push_active_token_with_arrival(instance, Some(edge_index), next_node_index);
    state::set_node_status(instance, next_node_index, NodeRuntimeStatus::Queued);
    arm_event_subprocess_waits(package, process, instance)?;
    state::record_transition(instance, now_ms, InstanceLifecycle::Running);
    Ok(())
}

fn event_subprocess_child<'a>(
    package: &'a BpmnPackage,
    parent_process: &BpmnProcessSpec,
    owner_node_index: BpmnNodeIndex,
) -> Result<(u32, &'a BpmnProcessSpec)> {
    let owner = parent_process.nodes.get(owner_node_index as usize).ok_or(
        BpmnEngineError::UnsupportedOperation {
            operation: "event_subprocess_missing_owner_node",
        },
    )?;
    if owner.subprocess_kind != Some(BpmnSubProcessKind::EventSubProcess) {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "event_subprocess_owner_kind_mismatch",
        });
    }
    let called_process_id =
        owner
            .called_process_id
            .as_ref()
            .ok_or(BpmnEngineError::UnsupportedOperation {
                operation: "event_subprocess_missing_child_process",
            })?;
    package
        .find_process_position(called_process_id.as_ref())
        .ok_or_else(|| BpmnEngineError::MissingProcess {
            process_id: called_process_id.to_string(),
        })
}
