use crate::test_support::MustExt as _;
use std::collections::HashMap;
use xiuxian_qianji_bpmn_engine::{BpmnFrontierEntryStatus, NodeRuntimeStatus, TokenRecord};

pub(super) const PROBE_DENSE_WAIT_ROLE_THRESHOLD: usize = 32;
pub(super) const PROBE_DENSE_WAIT_ROLE_TOKEN_THRESHOLD: usize = 128;
pub(super) const PROBE_DENSE_WAIT_ROLE_NODE_TO_TOKEN_RATIO: usize = 8;

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct ProbeWaitRole {
    pub(super) direct_wait: bool,
    pub(super) boundary_blocking: bool,
}

#[derive(Debug)]
pub(super) enum ProbeWaitRoleLookup {
    Empty,
    Sparse(HashMap<u32, ProbeWaitRole>),
    Dense(Vec<ProbeWaitRole>),
}

impl ProbeWaitRoleLookup {
    pub(super) fn new(
        active_token_count: usize,
        node_count: usize,
        waiting_node_indices: &[u32],
        boundary_blocking_node_indices: &[u32],
    ) -> Self {
        if waiting_node_indices.is_empty() && boundary_blocking_node_indices.is_empty() {
            return Self::Empty;
        }

        if should_probe_use_dense_wait_roles(
            active_token_count,
            node_count,
            waiting_node_indices.len() + boundary_blocking_node_indices.len(),
        ) {
            let mut wait_roles = vec![ProbeWaitRole::default(); node_count];
            for node_index in waiting_node_indices {
                if let Some(wait_role) = wait_roles.get_mut(*node_index as usize) {
                    wait_role.direct_wait = true;
                }
            }
            for node_index in boundary_blocking_node_indices {
                if let Some(wait_role) = wait_roles.get_mut(*node_index as usize) {
                    wait_role.boundary_blocking = true;
                }
            }
            return Self::Dense(wait_roles);
        }

        let mut wait_roles: HashMap<u32, ProbeWaitRole> = HashMap::with_capacity(
            waiting_node_indices.len() + boundary_blocking_node_indices.len(),
        );
        for node_index in waiting_node_indices {
            wait_roles.entry(*node_index).or_default().direct_wait = true;
        }
        for node_index in boundary_blocking_node_indices {
            wait_roles.entry(*node_index).or_default().boundary_blocking = true;
        }
        Self::Sparse(wait_roles)
    }

    pub(super) fn get(&self, node_index: u32) -> Option<ProbeWaitRole> {
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

impl ProbeWaitRole {
    fn is_active(self) -> bool {
        self.direct_wait || self.boundary_blocking
    }
}

pub(super) fn build_frontier_probe_node_statuses(node_count: u32) -> Vec<NodeRuntimeStatus> {
    (0..node_count)
        .map(|node_index| match node_index % 17 {
            0 => NodeRuntimeStatus::Queued,
            1 => NodeRuntimeStatus::Executing,
            2 => NodeRuntimeStatus::Completed,
            3 => NodeRuntimeStatus::Cancelled,
            4 => NodeRuntimeStatus::Failed,
            _ => NodeRuntimeStatus::Idle,
        })
        .collect()
}

pub(super) fn build_frontier_snapshot_probe_tokens(
    token_count: u64,
    node_count: u32,
) -> Vec<TokenRecord> {
    (0..token_count)
        .map(|offset| TokenRecord {
            token_id: offset + 1,
            node_index: 5 + u32::try_from(offset % u64::from(node_count - 5))
                .must("frontier snapshot probe token offset should fit in u32"),
            incoming_edge_index: Some(
                u32::try_from(offset % 8).must("frontier snapshot probe edge should fit in u32"),
            ),
            inclusive_join_hint: None,
        })
        .collect()
}

pub(super) fn build_frontier_snapshot_waiting_nodes(wait_count: u32) -> Vec<u32> {
    (0..wait_count).map(|offset| 100 + offset * 3).collect()
}

pub(super) fn build_frontier_snapshot_boundary_blocking_nodes(
    wait_count: u32,
    node_count: u32,
) -> Vec<u32> {
    (0..wait_count)
        .map(|offset| 5 + ((offset * 37) % (node_count - 5)))
        .collect()
}

pub(super) fn dense_frontier_status_for_node(
    status: &NodeRuntimeStatus,
) -> Option<BpmnFrontierEntryStatus> {
    match status {
        NodeRuntimeStatus::Queued => Some(BpmnFrontierEntryStatus::Runnable),
        NodeRuntimeStatus::Cancelled => Some(BpmnFrontierEntryStatus::Cancelled),
        NodeRuntimeStatus::Failed => Some(BpmnFrontierEntryStatus::Failed),
        NodeRuntimeStatus::Idle | NodeRuntimeStatus::Executing | NodeRuntimeStatus::Completed => {
            None
        }
    }
}

pub(super) fn direct_frontier_status_for_node(
    node_statuses: &[NodeRuntimeStatus],
    node_index: u32,
) -> Option<BpmnFrontierEntryStatus> {
    node_statuses
        .get(node_index as usize)
        .and_then(dense_frontier_status_for_node)
}

pub(super) fn terminal_frontier_status_for_node(
    status: &NodeRuntimeStatus,
) -> Option<BpmnFrontierEntryStatus> {
    match status {
        NodeRuntimeStatus::Cancelled => Some(BpmnFrontierEntryStatus::Cancelled),
        NodeRuntimeStatus::Failed => Some(BpmnFrontierEntryStatus::Failed),
        NodeRuntimeStatus::Idle
        | NodeRuntimeStatus::Queued
        | NodeRuntimeStatus::Executing
        | NodeRuntimeStatus::Completed => None,
    }
}

pub(super) fn frontier_status_code(status: BpmnFrontierEntryStatus) -> u64 {
    match status {
        BpmnFrontierEntryStatus::Runnable => 1,
        BpmnFrontierEntryStatus::BlockedOnHost => 2,
        BpmnFrontierEntryStatus::WaitingExternal => 3,
        BpmnFrontierEntryStatus::Cancelled => 4,
        BpmnFrontierEntryStatus::Failed => 5,
    }
}

fn should_probe_use_dense_wait_roles(
    active_token_count: usize,
    node_count: usize,
    wait_role_count: usize,
) -> bool {
    wait_role_count >= PROBE_DENSE_WAIT_ROLE_THRESHOLD
        && active_token_count >= PROBE_DENSE_WAIT_ROLE_TOKEN_THRESHOLD
        && node_count
            <= active_token_count.saturating_mul(PROBE_DENSE_WAIT_ROLE_NODE_TO_TOKEN_RATIO)
}
