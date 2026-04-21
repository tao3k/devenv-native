use super::competition::apply_event_competition_outcome;
use crate::BpmnEngineError;
use crate::error::Result;
use crate::ir::BpmnPackage;
use crate::ir_index_api::BpmnNodeIndex;
use crate::runtime::lifecycle::{
    merge_output_data, record_transition, resolve_single_outgoing_edge, set_active_node_index,
    set_node_status,
};
use crate::runtime::{
    BpmnAdvanceOutcome, BpmnInstanceState, InstanceLifecycle, NodeRuntimeStatus,
    parallel_multi_instance_min_token_id, resolve_process_for_instance,
};
use crate::{
    host_types_api::{EventPollOutcome, EventPollRequest},
    runtime_wait_api::WaitRegistration as RuntimeWaitRegistration,
};
use std::{borrow::Borrow, mem};

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
    let waits = wait_registrations(instance)?;
    Ok(EventPollRequest {
        instance_id: instance.instance_id.to_string(),
        gateway_node_index: instance
            .event_competition
            .as_ref()
            .map(|competition| competition.gateway_node_index),
        waits: waits.to_vec(),
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
    let waits = wait_registrations(instance)?.to_vec();

    if !outcome.ready {
        return Ok(BpmnAdvanceOutcome::WaitingExternalEvent);
    }

    let wait = resolve_winning_wait(&waits, outcome)?;
    let process = resolve_process_for_instance(package, instance)?;
    merge_output_data(&mut instance.variables, &outcome.data);

    if let Some(blocking_node_index) = wait.blocking_node_index {
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
        return Ok(BpmnAdvanceOutcome::Advanced);
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

fn wait_registrations(instance: &BpmnInstanceState) -> Result<&[RuntimeWaitRegistration]> {
    match instance.waits.as_slice() {
        [] => Err(BpmnEngineError::MissingWaitRegistration {
            instance_id: instance.instance_id.to_string(),
        }),
        waits => Ok(waits),
    }
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
