use super::competition::apply_event_competition_outcome;
use crate::BpmnEngineError;
use crate::error::Result;
use crate::ir::BpmnPackage;
use crate::ir_index_api::BpmnNodeIndex;
use crate::ir_process_spec::BpmnProcessSpec;
use crate::runtime::lifecycle::{
    cancel_attached_boundary_siblings, conditional_event_is_satisfied, merge_output_data,
    record_transition, resolve_single_outgoing_edge, set_active_node_index, set_node_status,
};
use crate::runtime::{
    BpmnAdvanceOutcome, BpmnInstanceState, InstanceLifecycle, NodeRuntimeStatus,
    parallel_multi_instance_min_token_id, push_active_token, resolve_process_for_instance,
};
use crate::{
    host_types_api::{EventPollOutcome, EventPollRequest},
    runtime_wait_api::WaitRegistration as RuntimeWaitRegistration,
};
use std::{borrow::Borrow, mem};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EventPollWaitSource {
    CurrentFrame,
    ParentFrame,
}

#[derive(Debug)]
struct EventPollWaitSet {
    source: EventPollWaitSource,
    gateway_node_index: Option<BpmnNodeIndex>,
    waits: Vec<RuntimeWaitRegistration>,
}

/// Builds one typed event-poll request from the current blocked wait state.
///
/// # Errors
///
/// Returns [`BpmnEngineError::MissingWaitRegistration`] when the instance does
/// not currently hold a wait registration, or [`BpmnEngineError`] when the
/// current wait shape exceeds the bounded single-wait slice.
pub(crate) fn build_event_poll_request_impl(
    instance: &BpmnInstanceState,
) -> Result<EventPollRequest> {
    let waits = event_poll_waits(instance)?;
    Ok(EventPollRequest {
        instance_id: instance.instance_id.to_string(),
        gateway_node_index: waits.gateway_node_index,
        waits: waits.waits,
    })
}

/// Applies one external-event poll outcome to the currently blocked instance.
///
/// # Errors
///
/// Returns [`BpmnEngineError::MissingWaitRegistration`] when the instance does
/// not currently hold a wait registration, or [`BpmnEngineError`] when the
/// current wait/runtime shape exceeds the bounded single-wait slice.
pub(crate) fn apply_event_poll_outcome_impl(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    outcome: impl Borrow<EventPollOutcome>,
    polled_at_ms: u64,
) -> Result<BpmnAdvanceOutcome> {
    let outcome = outcome.borrow();
    let wait_set = event_poll_waits(instance)?;

    let mut poll_data_merged = false;
    let wait = if outcome.ready || single_conditional_wait_candidate(&wait_set.waits) {
        resolve_winning_wait(&wait_set.waits, outcome)?
    } else if has_conditional_wait_candidate(&wait_set.waits) {
        merge_output_data(&mut instance.variables, &outcome.data);
        poll_data_merged = true;
        let Some(wait) = first_satisfied_conditional_wait(package, instance, &wait_set)? else {
            if outcome
                .data
                .as_object()
                .is_some_and(|object| !object.is_empty())
            {
                record_transition(instance, polled_at_ms, InstanceLifecycle::Waiting);
            }
            return Ok(BpmnAdvanceOutcome::WaitingExternalEvent);
        };
        wait
    } else {
        return Ok(BpmnAdvanceOutcome::WaitingExternalEvent);
    };

    let process = resolve_wait_process(package, instance, &wait, wait_set.source)?;
    if !poll_data_merged {
        merge_output_data(&mut instance.variables, &outcome.data);
    }
    if is_conditional_wait(&wait)
        && !conditional_event_is_satisfied(process, wait.node_index, &instance.variables)?
    {
        if outcome
            .data
            .as_object()
            .is_some_and(|object| !object.is_empty())
        {
            record_transition(instance, polled_at_ms, InstanceLifecycle::Waiting);
        }
        return Ok(BpmnAdvanceOutcome::WaitingExternalEvent);
    }

    if let Some(blocking_node_index) = wait.blocking_node_index {
        let boundary_node = &process.nodes[wait.node_index as usize];
        if boundary_node.cancel_activity {
            if wait_set.source == EventPollWaitSource::ParentFrame {
                return apply_interrupting_parent_frame_boundary_wait(
                    package,
                    instance,
                    &wait,
                    blocking_node_index,
                    polled_at_ms,
                );
            }
            return apply_interrupting_boundary_wait(
                process,
                instance,
                &wait,
                blocking_node_index,
                polled_at_ms,
            );
        }
        if wait_set.source == EventPollWaitSource::ParentFrame {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "apply_event_poll_outcome_parent_frame_non_interrupting_boundary",
            });
        }
        return apply_non_interrupting_boundary_wait(process, instance, &wait, polled_at_ms);
    }

    if wait_set.source == EventPollWaitSource::ParentFrame {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "apply_event_poll_outcome_parent_frame_standalone_wait",
        });
    }

    if let Some(competition) = instance.event_competition.clone() {
        return apply_event_competition_outcome(
            process,
            instance,
            &competition,
            &wait,
            polled_at_ms,
        );
    }

    let Some(wait_token_index) = active_token_index(instance, wait.node_index) else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "apply_event_poll_outcome_wait_missing_token",
        });
    };

    set_node_status(instance, wait.node_index, NodeRuntimeStatus::Completed);
    instance
        .waits
        .retain(|candidate| candidate.node_index != wait.node_index);
    if instance.waits.is_empty() {
        instance.suspend_reason = None;
    }

    let edge_index =
        resolve_single_outgoing_edge(process, wait.node_index, "apply_event_poll_outcome_routing")?;
    let next_node_index = process.edges[edge_index as usize].to;
    set_active_node_index(instance, wait_token_index, edge_index, next_node_index);
    set_node_status(instance, next_node_index, NodeRuntimeStatus::Queued);
    record_transition(instance, polled_at_ms, InstanceLifecycle::Running);

    Ok(BpmnAdvanceOutcome::Advanced)
}

fn single_conditional_wait_candidate(waits: &[RuntimeWaitRegistration]) -> bool {
    matches!(waits, [wait] if is_conditional_wait(wait))
}

fn has_conditional_wait_candidate(waits: &[RuntimeWaitRegistration]) -> bool {
    waits.iter().any(is_conditional_wait)
}

fn is_conditional_wait(wait: &RuntimeWaitRegistration) -> bool {
    wait.event_kind == Some(crate::ir_event_api::BpmnEventKind::Conditional)
}

fn first_satisfied_conditional_wait(
    package: &BpmnPackage,
    instance: &BpmnInstanceState,
    wait_set: &EventPollWaitSet,
) -> Result<Option<RuntimeWaitRegistration>> {
    for wait in wait_set
        .waits
        .iter()
        .filter(|wait| is_conditional_wait(wait))
    {
        let process = resolve_wait_process(package, instance, wait, wait_set.source)?;
        if conditional_event_is_satisfied(process, wait.node_index, &instance.variables)? {
            return Ok(Some(wait.clone()));
        }
    }
    Ok(None)
}

fn apply_interrupting_parent_frame_boundary_wait(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    wait: &RuntimeWaitRegistration,
    blocking_node_index: BpmnNodeIndex,
    polled_at_ms: u64,
) -> Result<BpmnAdvanceOutcome> {
    let frame = crate::runtime::pop_call_activity_frame(instance).ok_or(
        BpmnEngineError::UnsupportedOperation {
            operation: "apply_event_poll_outcome_parent_boundary_missing_frame",
        },
    )?;
    if frame.return_node_index != blocking_node_index {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "apply_event_poll_outcome_parent_boundary_owner_mismatch",
        });
    }

    let return_node_index = crate::runtime::restore_call_activity_frame(instance, frame);
    if return_node_index != blocking_node_index {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "apply_event_poll_outcome_parent_boundary_restore_mismatch",
        });
    }

    let process = resolve_process_for_instance(package, instance)?;
    if let Some(process_id) = wait.process_id.as_deref()
        && process.key.process_id.as_ref() != process_id
    {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "apply_event_poll_outcome_parent_boundary_process_mismatch",
        });
    }

    let Some(parent_token_index) = active_token_index(instance, blocking_node_index) else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "apply_event_poll_outcome_parent_boundary_missing_token",
        });
    };

    set_node_status(instance, blocking_node_index, NodeRuntimeStatus::Cancelled);
    cancel_attached_boundary_siblings(process, instance, blocking_node_index, &[wait.node_index])?;
    set_node_status(instance, wait.node_index, NodeRuntimeStatus::Completed);
    instance.waits.retain(|candidate| {
        candidate.node_index != wait.node_index
            && candidate.blocking_node_index != Some(blocking_node_index)
    });
    if instance.waits.is_empty() {
        instance.suspend_reason = None;
    }

    let edge_index = resolve_single_outgoing_edge(
        process,
        wait.node_index,
        "apply_event_poll_outcome_parent_boundary_routing",
    )?;
    let next_node_index = process.edges[edge_index as usize].to;
    set_active_node_index(instance, parent_token_index, edge_index, next_node_index);
    set_node_status(instance, next_node_index, NodeRuntimeStatus::Queued);
    record_transition(instance, polled_at_ms, InstanceLifecycle::Running);
    Ok(BpmnAdvanceOutcome::Advanced)
}

fn apply_interrupting_boundary_wait(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    wait: &RuntimeWaitRegistration,
    blocking_node_index: BpmnNodeIndex,
    polled_at_ms: u64,
) -> Result<BpmnAdvanceOutcome> {
    let Some(blocking_token_index) =
        interrupting_boundary_token_index(instance, blocking_node_index)
    else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "apply_event_poll_outcome_boundary_wait_missing_token",
        });
    };

    crate::runtime::clear_standard_loop_state(instance, blocking_node_index);
    crate::runtime::clear_sequential_multi_instance_state(instance, blocking_node_index);
    crate::runtime::clear_parallel_multi_instance_state(instance, blocking_node_index);
    set_node_status(instance, blocking_node_index, NodeRuntimeStatus::Cancelled);
    set_node_status(instance, wait.node_index, NodeRuntimeStatus::Completed);
    instance.waits.retain(|candidate| {
        candidate.node_index != wait.node_index
            && candidate.blocking_node_index != Some(blocking_node_index)
    });
    instance
        .pending_host_work
        .retain(|pending| pending.node_index != blocking_node_index);
    if instance.waits.is_empty() {
        instance.suspend_reason = None;
    }

    let edge_index = resolve_single_outgoing_edge(
        process,
        wait.node_index,
        "apply_event_poll_outcome_boundary_routing",
    )?;
    let next_node_index = process.edges[edge_index as usize].to;
    set_active_node_index(instance, blocking_token_index, edge_index, next_node_index);
    set_node_status(instance, next_node_index, NodeRuntimeStatus::Queued);
    record_transition(instance, polled_at_ms, InstanceLifecycle::Running);
    Ok(BpmnAdvanceOutcome::Advanced)
}

fn apply_non_interrupting_boundary_wait(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    wait: &RuntimeWaitRegistration,
    polled_at_ms: u64,
) -> Result<BpmnAdvanceOutcome> {
    set_node_status(instance, wait.node_index, NodeRuntimeStatus::Completed);
    instance
        .waits
        .retain(|candidate| candidate.node_index != wait.node_index);
    if instance.waits.is_empty() {
        instance.suspend_reason = None;
    }

    let edge_index = resolve_single_outgoing_edge(
        process,
        wait.node_index,
        "apply_event_poll_outcome_boundary_routing",
    )?;
    let next_node_index = process.edges[edge_index as usize].to;
    push_active_token(instance, edge_index, next_node_index);
    set_node_status(instance, next_node_index, NodeRuntimeStatus::Queued);
    record_transition(instance, polled_at_ms, InstanceLifecycle::Running);
    Ok(BpmnAdvanceOutcome::Advanced)
}

fn event_poll_waits(instance: &BpmnInstanceState) -> Result<EventPollWaitSet> {
    let parent_frame = instance
        .call_stack
        .last()
        .filter(|frame| !frame.waits.is_empty());
    match (instance.waits.is_empty(), parent_frame) {
        (false, None) => Ok(EventPollWaitSet {
            source: EventPollWaitSource::CurrentFrame,
            gateway_node_index: instance
                .event_competition
                .as_ref()
                .map(|competition| competition.gateway_node_index),
            waits: instance.waits.clone(),
        }),
        (true, Some(frame)) => Ok(EventPollWaitSet {
            source: EventPollWaitSource::ParentFrame,
            gateway_node_index: frame
                .event_competition
                .as_ref()
                .map(|competition| competition.gateway_node_index),
            waits: frame.waits.clone(),
        }),
        (false, Some(_)) => Err(BpmnEngineError::UnsupportedOperation {
            operation: "event_poll_waits_multiple_frame_levels",
        }),
        (true, None) => Err(BpmnEngineError::MissingWaitRegistration {
            instance_id: instance.instance_id.to_string(),
        }),
    }
}

fn resolve_wait_process<'a>(
    package: &'a BpmnPackage,
    instance: &BpmnInstanceState,
    wait: &RuntimeWaitRegistration,
    source: EventPollWaitSource,
) -> Result<&'a BpmnProcessSpec> {
    match source {
        EventPollWaitSource::CurrentFrame => {
            let process_id = wait
                .process_id
                .as_deref()
                .unwrap_or(instance.process.process_id.as_ref());
            resolve_wait_process_by_index(package, process_id, instance.process_index)
        }
        EventPollWaitSource::ParentFrame => {
            let Some(frame) = instance.call_stack.last() else {
                return Err(BpmnEngineError::UnsupportedOperation {
                    operation: "resolve_wait_process_missing_parent_frame",
                });
            };
            let process_id = wait
                .process_id
                .as_deref()
                .unwrap_or(frame.process.process_id.as_ref());
            resolve_wait_process_by_index(package, process_id, frame.process_index)
        }
    }
}

fn resolve_wait_process_by_index<'a>(
    package: &'a BpmnPackage,
    process_id: &str,
    process_index: u32,
) -> Result<&'a BpmnProcessSpec> {
    package
        .processes
        .get(process_index as usize)
        .filter(|process| process.key.process_id.as_ref() == process_id)
        .or_else(|| package.find_process(process_id))
        .ok_or_else(|| BpmnEngineError::MissingProcess {
            process_id: process_id.to_string(),
        })
}

fn resolve_winning_wait(
    waits: &[RuntimeWaitRegistration],
    outcome: &EventPollOutcome,
) -> Result<RuntimeWaitRegistration> {
    if waits.len() == 1 {
        let wait = waits[0].clone();
        if let Some(winning_wait_node_index) = outcome.winning_wait_node_index
            && winning_wait_node_index != wait.node_index
        {
            return Err(BpmnEngineError::UnsupportedOperation {
                operation: "apply_event_poll_outcome_single_wait_winner_mismatch",
            });
        }
        return Ok(wait);
    }

    let Some(winning_wait_node_index) = outcome.winning_wait_node_index else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "apply_event_poll_outcome_missing_competition_winner",
        });
    };
    waits
        .iter()
        .find(|wait| wait.node_index == winning_wait_node_index)
        .cloned()
        .ok_or(BpmnEngineError::UnsupportedOperation {
            operation: "apply_event_poll_outcome_unknown_competition_winner",
        })
}

fn active_token_index(instance: &BpmnInstanceState, node_index: BpmnNodeIndex) -> Option<usize> {
    instance
        .active_tokens
        .iter()
        .position(|token| token.node_index == node_index)
}

fn interrupting_boundary_token_index(
    instance: &mut BpmnInstanceState,
    blocking_node_index: BpmnNodeIndex,
) -> Option<usize> {
    let Some(winning_token_id) =
        parallel_multi_instance_min_token_id(instance, blocking_node_index)
    else {
        return active_token_index(instance, blocking_node_index);
    };

    retain_interrupting_boundary_winner_token(instance, blocking_node_index, winning_token_id)
}

fn retain_interrupting_boundary_winner_token(
    instance: &mut BpmnInstanceState,
    blocking_node_index: BpmnNodeIndex,
    winning_token_id: u64,
) -> Option<usize> {
    let mut winner_token_index = None;
    let mut surviving_tokens = Vec::with_capacity(instance.active_tokens.len());

    for token in mem::take(&mut instance.active_tokens) {
        if token.token_id == winning_token_id || token.node_index != blocking_node_index {
            if token.token_id == winning_token_id {
                winner_token_index = Some(surviving_tokens.len());
            }
            surviving_tokens.push(token);
        }
    }

    instance.active_tokens = surviving_tokens;
    winner_token_index
}
