//! Public runtime repeat and multi-instance state shells.

use crate::ir_index_api::BpmnNodeIndex;
use std::sync::Arc;

/// Snapshot of one active standard-loop owner.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StandardLoopState {
    /// Owning loop node index.
    pub node_index: BpmnNodeIndex,
    /// Completed iteration count.
    pub completed_iterations: u32,
}

/// Snapshot of one active sequential multi-instance owner.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SequentialMultiInstanceState {
    /// Owning multi-instance node index.
    pub node_index: BpmnNodeIndex,
    /// Total planned sequential iterations.
    pub total_iterations: u32,
    /// Completed iteration count.
    pub completed_iterations: u32,
    /// Optional checkpoint-safe collection binding state.
    #[serde(default)]
    pub data_binding: Option<MultiInstanceDataRuntimeState>,
}

/// Collection kind preserved for data-bound multi-instance expansion.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MultiInstanceCollectionKind {
    /// Iterations were expanded from one array.
    Array,
    /// Iterations were expanded from one object.
    Object,
}

/// Stable output key associated with one collection-backed iteration.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum MultiInstanceCollectionKey {
    /// Zero-based array position for one iteration.
    Index(u32),
    /// Object member key for one iteration.
    Key(Arc<str>),
}

/// Stable per-iteration input slot snapshot.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MultiInstanceCollectionSlot {
    /// Stable output key for the iteration.
    pub key: MultiInstanceCollectionKey,
    /// Snapshotted source item value for the iteration.
    pub input: serde_json::Value,
}

/// Aggregated output state for one collection-backed multi-instance owner.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MultiInstanceOutputCollectionState {
    /// Destination variable path for the aggregated collection.
    pub loop_data_output_ref: Arc<str>,
    /// Per-iteration output item name to collect.
    pub output_data_item: Arc<str>,
    /// Iteration-aligned output values written so far.
    #[serde(default)]
    pub values: Vec<Option<serde_json::Value>>,
}

/// Checkpoint-safe collection binding state for one multi-instance owner.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MultiInstanceDataRuntimeState {
    /// Source collection kind.
    pub collection_kind: MultiInstanceCollectionKind,
    /// Per-iteration input item variable name.
    pub input_data_item: Arc<str>,
    /// Stable iteration slots.
    #[serde(default)]
    pub slots: Vec<MultiInstanceCollectionSlot>,
    /// Optional output aggregation state.
    pub output: Option<MultiInstanceOutputCollectionState>,
}

/// Snapshot of one active parallel multi-instance iteration token.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParallelMultiInstanceIterationState {
    /// Runtime token that owns this active iteration.
    pub token_id: u64,
    /// Zero-based iteration index for this token.
    pub iteration_index: u32,
}

/// Snapshot of one active parallel multi-instance owner.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParallelMultiInstanceState {
    /// Owning multi-instance node index.
    pub node_index: BpmnNodeIndex,
    /// Total planned parallel iterations.
    pub total_iterations: u32,
    /// Completed iteration count.
    pub completed_iterations: u32,
    /// Optional checkpoint-safe collection binding state.
    #[serde(default)]
    pub data_binding: Option<MultiInstanceDataRuntimeState>,
    /// Active runtime tokens that still belong to this owner.
    #[serde(default)]
    pub active_iterations: Vec<ParallelMultiInstanceIterationState>,
}
