//! Runtime frontier coordination API.
//!
//! This module owns the public frontier planning contract while delegating
//! batch reduction and snapshot scanning to focused runtime/frontier leaves.

#[path = "runtime/frontier/batch.rs"]
mod batch;
#[path = "runtime/frontier/snapshot.rs"]
mod snapshot;

use crate::BpmnNodeIndex;
use crate::ir::BpmnProcessSpec;
use crate::runtime::{BpmnInstanceState, PendingHostWork, SuspendReason};

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

impl From<&BpmnFrontierEntry> for BpmnFrontierExecutionProposal {
    fn from(entry: &BpmnFrontierEntry) -> Self {
        Self {
            token_id: (entry.token_id),
            token_index: entry.token_index,
            node_index: entry.node_index,
            incoming_edge_index: entry.incoming_edge_index,
        }
    }
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

pub(crate) enum BpmnFrontierRuntimeAction {
    Execute(BpmnFrontierRuntimeBatch),
    BlockedOnHost(Vec<PendingHostWork>),
    WaitingExternalEvent,
    Suspended(Option<SuspendReason>),
    Stalled,
}

pub(crate) enum BpmnFrontierRuntimeBatch {
    Proposals(Vec<BpmnFrontierExecutionProposal>),
    Steps(Vec<BpmnFrontierExecutionStep>),
}

/// Planner output for one deterministic runtime step.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnFrontierPlan {
    /// Collected runnable proposals used to derive the next action.
    pub proposals: BpmnFrontierProposalSet,
    /// Deterministic next action for the owner process.
    pub action: BpmnFrontierPlanAction,
}

/// Materializes one immutable snapshot of the current runtime frontier.
#[must_use]
pub fn snapshot_frontier(instance: &BpmnInstanceState) -> BpmnFrontierSnapshot {
    BpmnFrontierSnapshot {
        entries: snapshot::snapshot_entries(instance),
    }
}

/// Collects runnable-token execution proposals from the current frontier.
#[must_use]
pub fn collect_frontier_proposals(instance: &BpmnInstanceState) -> BpmnFrontierProposalSet {
    let snapshot = snapshot_frontier(instance);
    let execution_proposals = snapshot
        .entries
        .iter()
        .filter(|entry| entry.status == BpmnFrontierEntryStatus::Runnable)
        .map(BpmnFrontierExecutionProposal::from)
        .collect();

    BpmnFrontierProposalSet {
        snapshot,
        execution_proposals,
    }
}

pub(crate) fn collect_frontier_execution_proposals(
    instance: &BpmnInstanceState,
) -> Vec<BpmnFrontierExecutionProposal> {
    snapshot::execution_proposals(instance)
}

/// Merges runnable frontier proposals into conflict-aware execution steps.
#[must_use]
pub fn merge_frontier_execution_steps(
    process: &BpmnProcessSpec,
    proposals: &[BpmnFrontierExecutionProposal],
) -> Vec<BpmnFrontierExecutionStep> {
    batch::merge_execution_steps(process, proposals)
}

/// Reduces collected frontier proposals to one deterministic owner action.
#[must_use]
pub fn reduce_frontier_plan(
    process: &BpmnProcessSpec,
    instance: &BpmnInstanceState,
    proposals: BpmnFrontierProposalSet,
) -> BpmnFrontierPlan {
    let action = reduce_frontier_action(process, instance, &proposals);
    BpmnFrontierPlan { proposals, action }
}

fn reduce_frontier_action(
    process: &BpmnProcessSpec,
    instance: &BpmnInstanceState,
    proposals: &BpmnFrontierProposalSet,
) -> BpmnFrontierPlanAction {
    if !proposals.execution_proposals.is_empty() {
        BpmnFrontierPlanAction::ExecuteBatch(batch::build_frontier_execution_batch(
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
    }
}

/// Plans the deterministic next runtime step from the current frontier.
#[must_use]
pub fn plan_frontier_step(
    process: &BpmnProcessSpec,
    instance: &BpmnInstanceState,
) -> BpmnFrontierPlan {
    reduce_frontier_plan(process, instance, collect_frontier_proposals(instance))
}

#[must_use]
pub(crate) fn plan_frontier_runtime_action(
    process: &BpmnProcessSpec,
    instance: &BpmnInstanceState,
) -> BpmnFrontierRuntimeAction {
    let execution_proposals = collect_frontier_execution_proposals(instance);
    if !execution_proposals.is_empty() {
        BpmnFrontierRuntimeAction::Execute(batch::build_frontier_runtime_batch(
            process,
            execution_proposals,
        ))
    } else if !instance.pending_host_work.is_empty() {
        BpmnFrontierRuntimeAction::BlockedOnHost(instance.pending_host_work.clone())
    } else if !instance.waits.is_empty() {
        BpmnFrontierRuntimeAction::WaitingExternalEvent
    } else if let Some(reason) = instance.suspend_reason.clone() {
        BpmnFrontierRuntimeAction::Suspended(Some(reason))
    } else {
        BpmnFrontierRuntimeAction::Stalled
    }
}
