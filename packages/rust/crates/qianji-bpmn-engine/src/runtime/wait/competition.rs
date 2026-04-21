use crate::BpmnEngineError;
use crate::error::Result;
use crate::ir::BpmnProcessSpec;
use crate::ir_index_api::BpmnNodeIndex;
use crate::runtime::lifecycle::{
    record_transition, resolve_single_outgoing_edge, set_active_node_index, set_node_status,
};
use crate::runtime::{
    BpmnAdvanceOutcome, BpmnInstanceState, EventCompetitionState, InstanceLifecycle,
    NodeRuntimeStatus, WaitRegistration,
};
use std::{collections::HashSet, mem};

const INDEXED_EVENT_COMPETITION_WAIT_THRESHOLD: usize = 4;

#[derive(Debug)]
struct EventCompetitionContext<'a> {
    wait_node_indices: &'a [BpmnNodeIndex],
    indexed_wait_node_indices: Option<HashSet<BpmnNodeIndex>>,
}

impl<'a> EventCompetitionContext<'a> {
    fn new(competition: &'a EventCompetitionState) -> Self {
        let wait_node_indices = competition.wait_node_indices.as_slice();
        let indexed_wait_node_indices = (wait_node_indices.len()
            >= INDEXED_EVENT_COMPETITION_WAIT_THRESHOLD)
            .then(|| wait_node_indices.iter().copied().collect());
        Self {
            wait_node_indices,
            indexed_wait_node_indices,
        }
    }

    fn contains_wait_node(&self, node_index: BpmnNodeIndex) -> bool {
        self.indexed_wait_node_indices.as_ref().map_or_else(
            || self.wait_node_indices.contains(&node_index),
            |indexed_wait_node_indices| indexed_wait_node_indices.contains(&node_index),
        )
    }

    fn retain_winner_token(
        &self,
        instance: &mut BpmnInstanceState,
        winner_token_id: u64,
    ) -> Option<usize> {
        let mut winner_token_index = None;
        let mut surviving_tokens = Vec::with_capacity(instance.active_tokens.len());

        for token in mem::take(&mut instance.active_tokens) {
            if token.token_id == winner_token_id || !self.contains_wait_node(token.node_index) {
                if token.token_id == winner_token_id {
                    winner_token_index = Some(surviving_tokens.len());
                }
                surviving_tokens.push(token);
            }
        }

        instance.active_tokens = surviving_tokens;
        winner_token_index
    }

    fn clear_competing_waits(&self, instance: &mut BpmnInstanceState) {
        instance
            .waits
            .retain(|wait| !self.contains_wait_node(wait.node_index));
    }
}

pub(super) fn apply_event_competition_outcome(
    process: &BpmnProcessSpec,
    instance: &mut BpmnInstanceState,
    competition: &EventCompetitionState,
    winning_wait: &WaitRegistration,
    polled_at_ms: u64,
) -> Result<BpmnAdvanceOutcome> {
    let context = EventCompetitionContext::new(competition);
    if !context.contains_wait_node(winning_wait.node_index) {
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
    let Some(winner_token_index) = context.retain_winner_token(instance, winner_token_id) else {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "apply_event_poll_outcome_competition_lost_winner_token",
        });
    };

    set_node_status(
        instance,
        winning_wait.node_index,
        NodeRuntimeStatus::Completed,
    );
    context.clear_competing_waits(instance);
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
