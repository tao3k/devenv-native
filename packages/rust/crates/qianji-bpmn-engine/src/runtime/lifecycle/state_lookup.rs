use super::token::token_index_for_id;
use crate::runtime::lifecycle::scope::{BpmnFrontierExecutionProposal, BpmnInstanceState};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub(crate) struct FrontierTokenLookup {
    token_indices: Option<HashMap<u64, usize>>,
    queries_since_refresh: u8,
}

impl FrontierTokenLookup {
    pub(crate) fn token_index_for_id(
        &mut self,
        instance: &BpmnInstanceState,
        token_id: u64,
    ) -> Option<usize> {
        if let Some(token_indices) = &self.token_indices {
            return token_indices.get(&token_id).copied();
        }

        self.queries_since_refresh = self.queries_since_refresh.saturating_add(1);
        if self.queries_since_refresh == 1 {
            return token_index_for_id(instance, token_id);
        }

        self.rebuild(instance);
        self.token_indices
            .as_ref()
            .and_then(|token_indices| token_indices.get(&token_id).copied())
    }

    pub(crate) fn resolve_frontier_proposal_token_index(
        &mut self,
        instance: &BpmnInstanceState,
        proposal: &BpmnFrontierExecutionProposal,
    ) -> Option<usize> {
        if proposal_matches_token_at_index(instance, proposal, proposal.token_index) {
            return Some(proposal.token_index);
        }

        let token_index = self.token_index_for_id(instance, proposal.token_id)?;
        proposal_matches_token_at_index(instance, proposal, token_index).then_some(token_index)
    }

    pub(crate) fn invalidate(&mut self) {
        self.token_indices = None;
        self.queries_since_refresh = 0;
    }

    fn rebuild(&mut self, instance: &BpmnInstanceState) {
        self.token_indices = Some(
            instance
                .active_tokens
                .iter()
                .enumerate()
                .map(|(token_index, token)| (token.token_id, token_index))
                .collect(),
        );
    }
}

fn proposal_matches_token_at_index(
    instance: &BpmnInstanceState,
    proposal: &BpmnFrontierExecutionProposal,
    token_index: usize,
) -> bool {
    instance
        .active_tokens
        .get(token_index)
        .is_some_and(|token| {
            token.token_id == proposal.token_id
                && token.node_index == proposal.node_index
                && token.incoming_edge_index == proposal.incoming_edge_index
        })
}
