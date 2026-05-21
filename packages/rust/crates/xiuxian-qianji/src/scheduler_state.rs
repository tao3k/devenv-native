//! Execution state tracking for Kahn's scheduling.

use crate::contracts::{NodeStatus, QianjiOutput};
use crate::engine::{QianjiEdge, QianjiEngine};
use crate::scheduler_preflight::resolve_wendao_placeholders_in_context;
use petgraph::Direction;
use petgraph::stable_graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

pub(crate) type NodeExecutionResult = std::result::Result<QianjiOutput, String>;

pub(crate) fn branch_label_matches(label: Option<&str>, active_branches: &HashSet<String>) -> bool {
    if let Some(value) = label {
        active_branches.contains(value)
    } else {
        true
    }
}

pub(crate) fn spawn_node_execution_task(
    engine_clone: Arc<RwLock<QianjiEngine>>,
    node_idx: NodeIndex,
    context_clone: serde_json::Value,
) -> tokio::task::JoinHandle<(NodeIndex, NodeExecutionResult)> {
    tokio::spawn(async move {
        let mechanism = {
            let mut engine = engine_clone.write().await;
            engine.graph[node_idx].status = NodeStatus::Executing;
            engine.graph[node_idx].mechanism.clone()
        };

        let result = match resolve_wendao_placeholders_in_context(&context_clone) {
            Ok(preflight_context) => mechanism.execute(&preflight_context).await,
            Err(error) => Err(error),
        };
        (node_idx, result)
    })
}

pub(crate) fn merge_output_data(context: &mut serde_json::Value, output_data: &serde_json::Value) {
    if let Some(obj) = output_data.as_object() {
        for (key, value) in obj {
            context[key] = value.clone();
        }
    }
}

/// Dynamic state for Kahn's topological execution.
pub struct ExecutionState {
    /// Queue of nodes ready to execute.
    pub ready_queue: VecDeque<NodeIndex>,
}

impl ExecutionState {
    pub(crate) fn build(engine: &QianjiEngine, active_branches: &HashSet<String>) -> Self {
        let ready_queue = engine
            .graph
            .node_indices()
            .filter(|node_idx| node_is_ready(engine, *node_idx, active_branches))
            .collect();
        Self { ready_queue }
    }
}

fn node_is_ready(
    engine: &QianjiEngine,
    node_idx: NodeIndex,
    active_branches: &HashSet<String>,
) -> bool {
    engine.graph[node_idx].status == NodeStatus::Idle
        && pending_dependency_count(engine, node_idx, active_branches) == 0
}

fn pending_dependency_count(
    engine: &QianjiEngine,
    node_idx: NodeIndex,
    active_branches: &HashSet<String>,
) -> usize {
    engine
        .graph
        .edges_directed(node_idx, Direction::Incoming)
        .filter(|edge| !incoming_dependency_is_satisfied(engine, edge, active_branches))
        .count()
}

fn incoming_dependency_is_satisfied(
    engine: &QianjiEngine,
    edge: &petgraph::stable_graph::EdgeReference<'_, QianjiEdge>,
    active_branches: &HashSet<String>,
) -> bool {
    let parent_done = engine.graph[edge.source()].status == NodeStatus::Completed;
    let branch_match = branch_label_matches(edge.weight().label.as_deref(), active_branches);
    parent_done && branch_match
}
