use super::{BpmnFrontierEntry, BpmnFrontierEntryStatus};
use crate::runtime::{BpmnInstanceState, NodeRuntimeStatus, TokenRecord};
use rayon::prelude::*;
use std::collections::HashSet;

const PARALLEL_FRONTIER_SCAN_THRESHOLD: usize = 32;

#[derive(Debug)]
struct FrontierScanContext {
    pending_token_ids: HashSet<u64>,
    waiting_node_indices: HashSet<crate::ir_index_api::BpmnNodeIndex>,
    boundary_blocking_node_indices: HashSet<crate::ir_index_api::BpmnNodeIndex>,
    queued_node_indices: HashSet<crate::ir_index_api::BpmnNodeIndex>,
    terminal_node_statuses: Vec<Option<BpmnFrontierEntryStatus>>,
}

impl FrontierScanContext {
    fn new(instance: &BpmnInstanceState) -> Self {
        let pending_token_ids = instance
            .pending_host_work
            .iter()
            .map(|pending| pending.token_id)
            .collect();
        let waiting_node_indices = instance.waits.iter().map(|wait| wait.node_index).collect();
        let boundary_blocking_node_indices = instance
            .waits
            .iter()
            .filter_map(|wait| wait.blocking_node_index)
            .collect();
        let queued_node_indices = instance
            .node_states
            .iter()
            .enumerate()
            .filter_map(|(node_index, state)| {
                if state.status != NodeRuntimeStatus::Queued {
                    return None;
                }
                crate::ir_index_api::BpmnNodeIndex::try_from(node_index).ok()
            })
            .collect();
        let terminal_node_statuses = instance
            .node_states
            .iter()
            .map(|state| match state.status {
                NodeRuntimeStatus::Cancelled => Some(BpmnFrontierEntryStatus::Cancelled),
                NodeRuntimeStatus::Failed => Some(BpmnFrontierEntryStatus::Failed),
                NodeRuntimeStatus::Idle
                | NodeRuntimeStatus::Queued
                | NodeRuntimeStatus::Executing
                | NodeRuntimeStatus::Completed => None,
            })
            .collect();

        Self {
            pending_token_ids,
            waiting_node_indices,
            boundary_blocking_node_indices,
            queued_node_indices,
            terminal_node_statuses,
        }
    }

    fn classify_token(&self, token: &TokenRecord) -> BpmnFrontierEntryStatus {
        if self.pending_token_ids.contains(&token.token_id) {
            return BpmnFrontierEntryStatus::BlockedOnHost;
        }
        if self.waiting_node_indices.contains(&token.node_index) {
            return BpmnFrontierEntryStatus::WaitingExternal;
        }
        if self
            .boundary_blocking_node_indices
            .contains(&token.node_index)
        {
            return if self.queued_node_indices.contains(&token.node_index) {
                BpmnFrontierEntryStatus::Runnable
            } else {
                BpmnFrontierEntryStatus::WaitingExternal
            };
        }
        self.terminal_node_statuses
            .get(token.node_index as usize)
            .and_then(|status| *status)
            .unwrap_or(BpmnFrontierEntryStatus::Runnable)
    }

    fn snapshot_token(&self, token_index: usize, token: &TokenRecord) -> BpmnFrontierEntry {
        BpmnFrontierEntry {
            token_id: token.token_id,
            token_index,
            node_index: token.node_index,
            incoming_edge_index: token.incoming_edge_index,
            status: self.classify_token(token),
        }
    }
}

pub(crate) fn snapshot_entries(instance: &BpmnInstanceState) -> Vec<BpmnFrontierEntry> {
    let context = FrontierScanContext::new(instance);
    if instance.active_tokens.len() >= PARALLEL_FRONTIER_SCAN_THRESHOLD {
        instance
            .active_tokens
            .par_iter()
            .enumerate()
            .map(|(token_index, token)| context.snapshot_token(token_index, token))
            .collect()
    } else {
        instance
            .active_tokens
            .iter()
            .enumerate()
            .map(|(token_index, token)| context.snapshot_token(token_index, token))
            .collect()
    }
}
