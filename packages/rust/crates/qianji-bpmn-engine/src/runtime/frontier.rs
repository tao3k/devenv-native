//! Deterministic token-frontier snapshots and planning helpers.

use super::{BpmnInstanceState, NodeRuntimeStatus, PendingHostWork, SuspendReason, TokenRecord};
use crate::ir::{BpmnGatewayKind, BpmnNodeIndex, BpmnNodeKind, BpmnProcessSpec};
use rayon::prelude::*;
use std::collections::HashSet;

const PARALLEL_FRONTIER_SCAN_THRESHOLD: usize = 32;

/// Classification for one active runtime token inside the current frontier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BpmnFrontierEntryStatus {
    /// The token is runnable in the current owner process.
    Runnable,
    /// The token is blocked on host-dispatched work.
    BlockedOnHost,
    /// The token is blocked on an external wait or boundary-owned wait.
    WaitingExternal,
    /// The token is attached to a cancelled node.
    Cancelled,
    /// The token is attached to a failed node.
    Failed,
}

/// Immutable frontier entry for one active runtime token.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnFrontierEntry {
    /// Owning runtime token identifier.
    pub token_id: u64,
    /// Current stable token position inside `active_tokens`.
    pub token_index: usize,
    /// Current BPMN node index.
    pub node_index: BpmnNodeIndex,
    /// Incoming sequence-flow edge that routed the token to this node.
    #[serde(default)]
    pub incoming_edge_index: Option<u32>,
    /// Current frontier classification.
    pub status: BpmnFrontierEntryStatus,
}

/// Immutable snapshot of the current instance frontier.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct BpmnFrontierSnapshot {
    /// Active token entries in deterministic token-index order.
    pub entries: Vec<BpmnFrontierEntry>,
}

impl BpmnFrontierSnapshot {
    /// Returns the first runnable frontier entry in deterministic token order.
    #[must_use]
    pub fn first_runnable_entry(&self) -> Option<&BpmnFrontierEntry> {
        self.entries
            .iter()
            .find(|entry| entry.status == BpmnFrontierEntryStatus::Runnable)
    }

    /// Returns the first runnable token position in deterministic frontier order.
    #[must_use]
    pub fn first_runnable_token_index(&self) -> Option<usize> {
        self.first_runnable_entry().map(|entry| entry.token_index)
    }
}

/// Execution proposal for one runnable token in the current frontier.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnFrontierExecutionProposal {
    /// Owning runtime token identifier.
    pub token_id: u64,
    /// Current stable token position inside `active_tokens`.
    pub token_index: usize,
    /// Current BPMN node index.
    pub node_index: BpmnNodeIndex,
    /// Incoming sequence-flow edge that routed the token to this node.
    #[serde(default)]
    pub incoming_edge_index: Option<u32>,
}

/// Collected runnable-token proposals for one immutable frontier snapshot.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnFrontierProposalSet {
    /// Snapshot that proposal collection was derived from.
    pub snapshot: BpmnFrontierSnapshot,
    /// Runnable execution proposals in deterministic token order.
    pub execution_proposals: Vec<BpmnFrontierExecutionProposal>,
}

impl BpmnFrontierProposalSet {
    /// Returns the first execution proposal in deterministic frontier order.
    #[must_use]
    pub fn first_execution_proposal(&self) -> Option<&BpmnFrontierExecutionProposal> {
        self.execution_proposals.first()
    }
}

/// Conflict-aware execution group for multiple proposals that target one
/// parallel join.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnFrontierParallelJoinMerge {
    /// Shared join-node index.
    pub node_index: BpmnNodeIndex,
    /// Merged proposals in deterministic frontier order.
    pub proposals: Vec<BpmnFrontierExecutionProposal>,
}

/// One executable step within a conflict-aware frontier batch.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BpmnFrontierExecutionStep {
    /// Execute one proposal without any node-family merge.
    Proposal(BpmnFrontierExecutionProposal),
    /// Execute one merged set of parallel-join arrivals.
    ParallelJoin(BpmnFrontierParallelJoinMerge),
}

/// Conflict-aware execution batch derived from the runnable frontier.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnFrontierExecutionBatch {
    /// Raw runnable proposals preserved in deterministic frontier order.
    pub proposals: Vec<BpmnFrontierExecutionProposal>,
    /// Conflict-aware execution steps derived from those proposals.
    pub steps: Vec<BpmnFrontierExecutionStep>,
}

/// Deterministic next-step action planned from one immutable frontier snapshot.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BpmnFrontierPlanAction {
    /// Execute the runnable proposals from the current frontier batch.
    ExecuteBatch(BpmnFrontierExecutionBatch),
    /// Return blocked-on-host because no runnable tokens exist.
    BlockedOnHost(Vec<PendingHostWork>),
    /// Return waiting-on-external-event because no runnable tokens exist.
    WaitingExternalEvent,
    /// Return suspended because no runnable tokens exist.
    Suspended(Option<SuspendReason>),
    /// No runnable token or idle outcome could be derived from the frontier.
    Stalled,
}

/// Planner output for one deterministic runtime step.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnFrontierPlan {
    /// Collected runnable proposals used to derive the next action.
    pub proposals: BpmnFrontierProposalSet,
    /// Deterministic next action for the owner process.
    pub action: BpmnFrontierPlanAction,
}

#[derive(Debug)]
struct FrontierScanContext {
    pending_token_ids: HashSet<u64>,
    waiting_node_indices: HashSet<BpmnNodeIndex>,
    boundary_blocking_node_indices: HashSet<BpmnNodeIndex>,
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
            terminal_node_statuses,
        }
    }

    fn classify_token(&self, token: &TokenRecord) -> BpmnFrontierEntryStatus {
        if self.pending_token_ids.contains(&token.token_id) {
            return BpmnFrontierEntryStatus::BlockedOnHost;
        }
        if self.waiting_node_indices.contains(&token.node_index)
            || self
                .boundary_blocking_node_indices
                .contains(&token.node_index)
        {
            return BpmnFrontierEntryStatus::WaitingExternal;
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

/// Materializes one immutable snapshot of the current runtime frontier.
#[must_use]
pub fn snapshot_frontier(instance: &BpmnInstanceState) -> BpmnFrontierSnapshot {
    let context = FrontierScanContext::new(instance);
    let entries = if instance.active_tokens.len() >= PARALLEL_FRONTIER_SCAN_THRESHOLD {
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
    };
    BpmnFrontierSnapshot { entries }
}

/// Collects runnable-token execution proposals from the current frontier.
#[must_use]
pub fn collect_frontier_proposals(instance: &BpmnInstanceState) -> BpmnFrontierProposalSet {
    let snapshot = snapshot_frontier(instance);
    let execution_proposals = snapshot
        .entries
        .iter()
        .filter(|entry| entry.status == BpmnFrontierEntryStatus::Runnable)
        .map(|entry| BpmnFrontierExecutionProposal {
            token_id: entry.token_id,
            token_index: entry.token_index,
            node_index: entry.node_index,
            incoming_edge_index: entry.incoming_edge_index,
        })
        .collect();

    BpmnFrontierProposalSet {
        snapshot,
        execution_proposals,
    }
}

fn is_parallel_join_node(process: &BpmnProcessSpec, node_index: BpmnNodeIndex) -> bool {
    let node = &process.nodes[node_index as usize];
    node.kind == BpmnNodeKind::Gateway
        && node.gateway_kind == Some(BpmnGatewayKind::Parallel)
        && process.incoming_edge_indices(node_index).len() > 1
}

/// Merges runnable frontier proposals into conflict-aware execution steps.
#[must_use]
pub fn merge_frontier_execution_steps(
    process: &BpmnProcessSpec,
    proposals: &[BpmnFrontierExecutionProposal],
) -> Vec<BpmnFrontierExecutionStep> {
    let mut steps = Vec::new();
    let mut index = 0;
    while index < proposals.len() {
        let proposal = &proposals[index];
        if is_parallel_join_node(process, proposal.node_index) {
            let mut merged = vec![proposal.clone()];
            index += 1;
            while index < proposals.len()
                && proposals[index].node_index == proposal.node_index
                && is_parallel_join_node(process, proposals[index].node_index)
            {
                merged.push(proposals[index].clone());
                index += 1;
            }
            if merged.len() == 1 {
                if let Some(only_proposal) = merged.pop() {
                    steps.push(BpmnFrontierExecutionStep::Proposal(only_proposal));
                }
            } else {
                steps.push(BpmnFrontierExecutionStep::ParallelJoin(
                    BpmnFrontierParallelJoinMerge {
                        node_index: proposal.node_index,
                        proposals: merged,
                    },
                ));
            }
            continue;
        }

        steps.push(BpmnFrontierExecutionStep::Proposal(proposal.clone()));
        index += 1;
    }
    steps
}

fn build_frontier_execution_batch(
    process: &BpmnProcessSpec,
    proposals: Vec<BpmnFrontierExecutionProposal>,
) -> BpmnFrontierExecutionBatch {
    let steps = merge_frontier_execution_steps(process, &proposals);
    BpmnFrontierExecutionBatch { proposals, steps }
}

/// Reduces collected frontier proposals to one deterministic owner action.
#[must_use]
pub fn reduce_frontier_plan(
    process: &BpmnProcessSpec,
    instance: &BpmnInstanceState,
    proposals: BpmnFrontierProposalSet,
) -> BpmnFrontierPlan {
    let action = if !proposals.execution_proposals.is_empty() {
        BpmnFrontierPlanAction::ExecuteBatch(build_frontier_execution_batch(
            process,
            proposals.execution_proposals.clone(),
        ))
    } else if !instance.pending_host_work.is_empty() {
        BpmnFrontierPlanAction::BlockedOnHost(instance.pending_host_work.clone())
    } else if !instance.waits.is_empty() {
        BpmnFrontierPlanAction::WaitingExternalEvent
    } else if let Some(reason) = instance.suspend_reason.clone() {
        BpmnFrontierPlanAction::Suspended(Some(reason))
    } else {
        BpmnFrontierPlanAction::Stalled
    };

    BpmnFrontierPlan { proposals, action }
}

/// Plans the deterministic next runtime step from the current frontier.
#[must_use]
pub fn plan_frontier_step(
    process: &BpmnProcessSpec,
    instance: &BpmnInstanceState,
) -> BpmnFrontierPlan {
    reduce_frontier_plan(process, instance, collect_frontier_proposals(instance))
}
