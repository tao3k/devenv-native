use super::frontier_snapshot_data::{
    ProbeWaitRole, ProbeWaitRoleLookup, dense_frontier_status_for_node,
    direct_frontier_status_for_node, frontier_status_code, terminal_frontier_status_for_node,
};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnFrontierEntryStatus, NodeRuntimeStatus, TokenRecord};
use std::collections::{HashMap, HashSet};

pub(super) fn hashset_frontier_snapshot_classification_sum(
    active_tokens: &[TokenRecord],
    node_statuses: &[NodeRuntimeStatus],
    waiting_node_indices: &[u32],
    boundary_blocking_node_indices: &[u32],
) -> u64 {
    let pending_token_ids = HashSet::<u64>::new();
    let waiting_node_indices: HashSet<u32> = waiting_node_indices.iter().copied().collect();
    let boundary_blocking_node_indices: HashSet<u32> =
        boundary_blocking_node_indices.iter().copied().collect();
    let queued_node_indices: HashSet<u32> = node_statuses
        .iter()
        .enumerate()
        .filter_map(|(node_index, status)| {
            (status == &NodeRuntimeStatus::Queued)
                .then_some(u32::try_from(node_index).must("node index should fit in u32"))
        })
        .collect();
    let terminal_node_statuses = node_statuses
        .iter()
        .map(terminal_frontier_status_for_node)
        .collect::<Vec<_>>();

    active_tokens
        .iter()
        .map(|token| {
            let status = if pending_token_ids.contains(&token.token_id) {
                BpmnFrontierEntryStatus::BlockedOnHost
            } else if waiting_node_indices.contains(&token.node_index) {
                BpmnFrontierEntryStatus::WaitingExternal
            } else if boundary_blocking_node_indices.contains(&token.node_index) {
                if queued_node_indices.contains(&token.node_index) {
                    BpmnFrontierEntryStatus::Runnable
                } else {
                    BpmnFrontierEntryStatus::WaitingExternal
                }
            } else {
                terminal_node_statuses
                    .get(token.node_index as usize)
                    .and_then(|status| *status)
                    .unwrap_or(BpmnFrontierEntryStatus::Runnable)
            };
            frontier_status_code(status)
        })
        .sum()
}

pub(super) fn dense_frontier_snapshot_classification_sum(
    active_tokens: &[TokenRecord],
    node_statuses: &[NodeRuntimeStatus],
    waiting_node_indices: &[u32],
    boundary_blocking_node_indices: &[u32],
) -> u64 {
    let pending_token_ids = HashSet::<u64>::new();
    let waiting_node_indices: HashSet<u32> = waiting_node_indices.iter().copied().collect();
    let boundary_blocking_node_indices: HashSet<u32> =
        boundary_blocking_node_indices.iter().copied().collect();
    let node_frontier_statuses = node_statuses
        .iter()
        .map(dense_frontier_status_for_node)
        .collect::<Vec<_>>();

    active_tokens
        .iter()
        .map(|token| {
            let status = if pending_token_ids.contains(&token.token_id) {
                BpmnFrontierEntryStatus::BlockedOnHost
            } else if waiting_node_indices.contains(&token.node_index) {
                BpmnFrontierEntryStatus::WaitingExternal
            } else if boundary_blocking_node_indices.contains(&token.node_index) {
                if node_frontier_statuses
                    .get(token.node_index as usize)
                    .and_then(|status| *status)
                    == Some(BpmnFrontierEntryStatus::Runnable)
                {
                    BpmnFrontierEntryStatus::Runnable
                } else {
                    BpmnFrontierEntryStatus::WaitingExternal
                }
            } else {
                node_frontier_statuses
                    .get(token.node_index as usize)
                    .and_then(|status| *status)
                    .unwrap_or(BpmnFrontierEntryStatus::Runnable)
            };
            frontier_status_code(status)
        })
        .sum()
}

pub(super) fn direct_frontier_snapshot_classification_sum(
    active_tokens: &[TokenRecord],
    node_statuses: &[NodeRuntimeStatus],
    waiting_node_indices: &[u32],
    boundary_blocking_node_indices: &[u32],
) -> u64 {
    let pending_token_ids = HashSet::<u64>::new();
    let mut wait_roles_by_node =
        HashMap::<u32, ProbeWaitRole>::with_capacity(waiting_node_indices.len() * 2);
    for node_index in waiting_node_indices {
        wait_roles_by_node
            .entry(*node_index)
            .or_default()
            .direct_wait = true;
    }
    for node_index in boundary_blocking_node_indices {
        wait_roles_by_node
            .entry(*node_index)
            .or_default()
            .boundary_blocking = true;
    }

    active_tokens
        .iter()
        .map(|token| {
            let status = if pending_token_ids.contains(&token.token_id) {
                BpmnFrontierEntryStatus::BlockedOnHost
            } else if let Some(wait_role) = wait_roles_by_node.get(&token.node_index) {
                status_for_wait_role(*wait_role, node_statuses, token.node_index)
            } else {
                direct_frontier_status_for_node(node_statuses, token.node_index)
                    .unwrap_or(BpmnFrontierEntryStatus::Runnable)
            };
            frontier_status_code(status)
        })
        .sum()
}

pub(super) fn adaptive_frontier_snapshot_classification_sum(
    active_tokens: &[TokenRecord],
    node_statuses: &[NodeRuntimeStatus],
    waiting_node_indices: &[u32],
    boundary_blocking_node_indices: &[u32],
) -> u64 {
    let pending_token_ids = HashSet::<u64>::new();
    let wait_roles = ProbeWaitRoleLookup::new(
        active_tokens.len(),
        node_statuses.len(),
        waiting_node_indices,
        boundary_blocking_node_indices,
    );

    active_tokens
        .iter()
        .map(|token| {
            let status = if pending_token_ids.contains(&token.token_id) {
                BpmnFrontierEntryStatus::BlockedOnHost
            } else if let Some(wait_role) = wait_roles.get(token.node_index) {
                status_for_wait_role(wait_role, node_statuses, token.node_index)
            } else {
                direct_frontier_status_for_node(node_statuses, token.node_index)
                    .unwrap_or(BpmnFrontierEntryStatus::Runnable)
            };
            frontier_status_code(status)
        })
        .sum()
}

fn status_for_wait_role(
    wait_role: ProbeWaitRole,
    node_statuses: &[NodeRuntimeStatus],
    node_index: u32,
) -> BpmnFrontierEntryStatus {
    if wait_role.direct_wait {
        BpmnFrontierEntryStatus::WaitingExternal
    } else if wait_role.boundary_blocking {
        if direct_frontier_status_for_node(node_statuses, node_index)
            == Some(BpmnFrontierEntryStatus::Runnable)
        {
            BpmnFrontierEntryStatus::Runnable
        } else {
            BpmnFrontierEntryStatus::WaitingExternal
        }
    } else {
        direct_frontier_status_for_node(node_statuses, node_index)
            .unwrap_or(BpmnFrontierEntryStatus::Runnable)
    }
}
