//! Waiting-state runtime records and bounded event-poll helpers.

use crate::ir_index_api::BpmnNodeIndex;
use crate::runtime_wait_api::WaitRegistration;
use crate::{
    BpmnEngineError,
    error::Result,
    host_types_api::{EventPollOutcome, EventPollRequest},
    ir::BpmnPackage,
};
use std::borrow::Borrow;

use super::{
    BpmnAdvanceOutcome, BpmnInstanceState, InstanceLifecycle, NodeRuntimeStatus,
    clear_parallel_multi_instance_state, clear_sequential_multi_instance_state,
    clear_standard_loop_state,
    lifecycle::{
        merge_output_data, record_transition, resolve_single_outgoing_edge, set_active_node_index,
        set_node_status,
    },
    parallel_multi_instance_token_ids, resolve_process_for_instance,
};

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

    let wait = resolve_winning_wait(instance, &waits, outcome)?;
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

        clear_standard_loop_state(instance, blocking_node_index);
        clear_sequential_multi_instance_state(instance, blocking_node_index);
        clear_parallel_multi_instance_state(instance, blocking_node_index);
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

fn wait_registrations(instance: &BpmnInstanceState) -> Result<&[WaitRegistration]> {
    match instance.waits.as_slice() {
        [] => Err(BpmnEngineError::MissingWaitRegistration {
            instance_id: instance.instance_id.to_string(),
        }),
        waits => Ok(waits),
    }
}

fn resolve_winning_wait(
    _instance: &BpmnInstanceState,
    waits: &[WaitRegistration],
    outcome: &EventPollOutcome,
) -> Result<WaitRegistration> {
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

fn apply_event_competition_outcome(
    process: &crate::ir::BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    competition: &super::EventCompetitionState,
    winning_wait: &WaitRegistration,
    polled_at_ms: u64,
) -> Result<BpmnAdvanceOutcome> {
    if !competition
        .wait_node_indices
        .contains(&winning_wait.node_index)
    {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "apply_event_poll_outcome_competition_winner_outside_owner",
        });
    }

    let Some(winning_token_index) = active_token_index(instance, winning_wait.node_index) else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "apply_event_poll_outcome_competition_missing_winner_token",
        });
    };

    for wait_node_index in &competition.wait_node_indices {
        if *wait_node_index == winning_wait.node_index {
            continue;
        }
        set_node_status(instance, *wait_node_index, NodeRuntimeStatus::Cancelled);
    }

    let winner_token_id = instance.active_tokens[winning_token_index].token_id;
    instance.active_tokens.retain(|token| {
        token.token_id == winner_token_id
            || !competition.wait_node_indices.contains(&token.node_index)
    });

    let Some(winner_token_index) = instance
        .active_tokens
        .iter()
        .position(|token| token.token_id == winner_token_id)
    else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "apply_event_poll_outcome_competition_lost_winner_token",
        });
    };

    set_node_status(
        instance,
        winning_wait.node_index,
        NodeRuntimeStatus::Completed,
    );
    instance
        .waits
        .retain(|wait| !competition.wait_node_indices.contains(&wait.node_index));
    instance.event_competition = None;
    if instance.waits.is_empty() {
        instance.suspend_reason = None;
    }

    let edge_index = resolve_single_outgoing_edge(
        process,
        winning_wait.node_index,
        "apply_event_poll_outcome_event_gateway_routing",
    )?;
    let next_node_index = process.edges[edge_index as usize].to;
    set_active_node_index(instance, winner_token_index, edge_index, next_node_index);
    set_node_status(instance, next_node_index, NodeRuntimeStatus::Queued);
    record_transition(instance, polled_at_ms, InstanceLifecycle::Running);

    Ok(BpmnAdvanceOutcome::Advanced)
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
    let parallel_token_ids = parallel_multi_instance_token_ids(instance, blocking_node_index);
    if parallel_token_ids.is_empty() {
        return active_token_index(instance, blocking_node_index);
    }

    let winning_token_id = parallel_token_ids.into_iter().min()?;
    instance.active_tokens.retain(|token| {
        token.token_id == winning_token_id || token.node_index != blocking_node_index
    });
    instance
        .active_tokens
        .iter()
        .position(|token| token.token_id == winning_token_id)
}
