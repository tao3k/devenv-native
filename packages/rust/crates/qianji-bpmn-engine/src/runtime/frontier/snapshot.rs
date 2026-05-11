use super::{BpmnFrontierEntry, BpmnFrontierEntryStatus, BpmnFrontierExecutionProposal};
use crate::runtime::{BpmnInstanceState, NodeRuntimeStatus, TokenRecord};
use crate::runtime_instance_api::NodeRuntimeState;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::collections::{HashMap, HashSet};

const PARALLEL_FRONTIER_SCAN_THRESHOLD: usize = 32;
const DENSE_WAIT_ROLE_THRESHOLD: usize = 32;
const DENSE_WAIT_ROLE_ACTIVE_TOKEN_THRESHOLD: usize = 128;
const DENSE_WAIT_ROLE_NODE_TO_TOKEN_RATIO: usize = 8;

#[derive(Debug)]
struct FrontierScanContext<'a> {
    pending_token_ids: PendingTokenLookup,
    wait_roles: WaitRoleLookup,
    node_states: &'a [NodeRuntimeState],
}

#[derive(Debug)]
enum PendingTokenLookup {
    Empty,
    Sparse(HashSet<u64>),
}

#[derive(Debug, Clone, Copy, Default)]
struct WaitFrontierRole {
    direct_wait: bool,
    boundary_blocking: bool,
}

#[derive(Debug)]
enum WaitRoleLookup {
    Empty,
    Sparse(HashMap<crate::ir_index_api::BpmnNodeIndex, WaitFrontierRole>),
    Dense(Vec<WaitFrontierRole>),
}

impl WaitRoleLookup {
    fn new(instance: &BpmnInstanceState) -> Self {
        if instance.waits.is_empty() {
            return Self::Empty;
        }

        if should_use_dense_wait_roles(instance) {
            let mut wait_roles = vec![WaitFrontierRole::default(); instance.node_states.len()];
            for wait in &instance.waits {
                if let Some(wait_role) = wait_roles.get_mut(wait.node_index as usize) {
                    wait_role.direct_wait = true;
                }
                if let Some(blocking_node_index) = wait.blocking_node_index
                    && let Some(wait_role) = wait_roles.get_mut(blocking_node_index as usize)
                {
                    wait_role.boundary_blocking = true;
                }
            }
            return Self::Dense(wait_roles);
        }

        let mut wait_roles: HashMap<crate::ir_index_api::BpmnNodeIndex, WaitFrontierRole> =
            HashMap::with_capacity(instance.waits.len() * 2);
        for wait in &instance.waits {
            wait_roles.entry(wait.node_index).or_default().direct_wait = true;
            if let Some(blocking_node_index) = wait.blocking_node_index {
                wait_roles
                    .entry(blocking_node_index)
                    .or_default()
                    .boundary_blocking = true;
            }
        }
        Self::Sparse(wait_roles)
    }

    fn get(&self, node_index: crate::ir_index_api::BpmnNodeIndex) -> Option<WaitFrontierRole> {
        match self {
            Self::Empty => None,
            Self::Sparse(wait_roles) => wait_roles.get(&node_index).copied(),
            Self::Dense(wait_roles) => wait_roles
                .get(node_index as usize)
                .copied()
                .filter(|role| role.is_active()),
        }
    }
}

impl WaitFrontierRole {
    fn is_active(self) -> bool {
        self.direct_wait || self.boundary_blocking
    }
}

fn should_use_dense_wait_roles(instance: &BpmnInstanceState) -> bool {
    instance.waits.len() >= DENSE_WAIT_ROLE_THRESHOLD
        && instance.active_tokens.len() >= DENSE_WAIT_ROLE_ACTIVE_TOKEN_THRESHOLD
        && instance.node_states.len()
            <= instance
                .active_tokens
                .len()
                .saturating_mul(DENSE_WAIT_ROLE_NODE_TO_TOKEN_RATIO)
}

impl<'a> FrontierScanContext<'a> {
    fn new(instance: &'a BpmnInstanceState) -> Self {
        Self {
            pending_token_ids: PendingTokenLookup::new(instance),
            wait_roles: WaitRoleLookup::new(instance),
            node_states: &instance.node_states,
        }
    }

    fn classify_token(&self, token: &TokenRecord) -> BpmnFrontierEntryStatus {
        if self.pending_token_ids.contains(token.token_id) {
            return BpmnFrontierEntryStatus::BlockedOnHost;
        }
        if let Some(wait_role) = self.wait_roles.get(token.node_index) {
            if wait_role.direct_wait {
                return BpmnFrontierEntryStatus::WaitingExternal;
            }
            if wait_role.boundary_blocking {
                return if self.node_status(token.node_index)
                    == Some(BpmnFrontierEntryStatus::Runnable)
                {
                    BpmnFrontierEntryStatus::Runnable
                } else {
                    BpmnFrontierEntryStatus::WaitingExternal
                };
            }
        }
        self.node_status(token.node_index)
            .unwrap_or(BpmnFrontierEntryStatus::Runnable)
    }

    fn node_status(
        &self,
        node_index: crate::ir_index_api::BpmnNodeIndex,
    ) -> Option<BpmnFrontierEntryStatus> {
        self.node_states
            .get(node_index as usize)
            .and_then(|state| match state.status {
                NodeRuntimeStatus::Queued => Some(BpmnFrontierEntryStatus::Runnable),
                NodeRuntimeStatus::Cancelled => Some(BpmnFrontierEntryStatus::Cancelled),
                NodeRuntimeStatus::Failed => Some(BpmnFrontierEntryStatus::Failed),
                NodeRuntimeStatus::Idle
                | NodeRuntimeStatus::Executing
                | NodeRuntimeStatus::Completed => None,
            })
    }

    fn snapshot_token(&self, token_index: usize, token: &TokenRecord) -> BpmnFrontierEntry {
        BpmnFrontierEntry {
            token_id: (token.token_id).into(),
            token_index,
            node_index: token.node_index,
            incoming_edge_index: token.incoming_edge_index,
            status: self.classify_token(token),
        }
    }

    fn execution_proposal(
        &self,
        token_index: usize,
        token: &TokenRecord,
    ) -> Option<BpmnFrontierExecutionProposal> {
        (self.classify_token(token) == BpmnFrontierEntryStatus::Runnable).then_some(
            BpmnFrontierExecutionProposal {
                token_id: (token.token_id).into(),
                token_index,
                node_index: token.node_index,
                incoming_edge_index: token.incoming_edge_index,
            },
        )
    }
}

impl PendingTokenLookup {
    fn new(instance: &BpmnInstanceState) -> Self {
        if instance.pending_host_work.is_empty() {
            return Self::Empty;
        }
        Self::Sparse(
            instance
                .pending_host_work
                .iter()
                .map(|pending| pending.token_id)
                .collect(),
        )
    }

    fn contains(&self, token_id: u64) -> bool {
        match self {
            Self::Empty => false,
            Self::Sparse(pending_token_ids) => pending_token_ids.contains(&token_id),
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

pub(super) fn execution_proposals(
    instance: &BpmnInstanceState,
) -> Vec<BpmnFrontierExecutionProposal> {
    let context = FrontierScanContext::new(instance);
    if instance.active_tokens.len() >= PARALLEL_FRONTIER_SCAN_THRESHOLD {
        instance
            .active_tokens
            .par_iter()
            .enumerate()
            .filter_map(|(token_index, token)| context.execution_proposal(token_index, token))
            .collect()
    } else {
        instance
            .active_tokens
            .iter()
            .enumerate()
            .filter_map(|(token_index, token)| context.execution_proposal(token_index, token))
            .collect()
    }
}
