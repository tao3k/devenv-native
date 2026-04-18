//! Execution state tracking for Kahn's scheduling.

use crate::contracts::{NodeStatus, QianjiOutput};
use crate::engine::QianjiEngine;
use crate::scheduler::preflight::resolve_wendao_placeholders_in_context;
use petgraph::Direction;
use petgraph::stable_graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet, VecDeque};
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
    /// Tracks unmet dependencies for each node.
    pub in_degrees: HashMap<NodeIndex, usize>,
    /// Queue of nodes ready to execute.
    pub ready_queue: VecDeque<NodeIndex>,
}

impl ExecutionState {
    pub(crate) fn build(engine: &QianjiEngine, active_branches: &HashSet<String>) -> Self {
        let mut in_degrees = HashMap::new();
        let mut ready_queue = VecDeque::new();

        for node_idx in engine.graph.node_indices() {
            if engine.graph[node_idx].status == NodeStatus::Idle {
                let mut degree = 0;
                for edge in engine.graph.edges_directed(node_idx, Direction::Incoming) {
                    let parent_idx = edge.source();
                    let edge_data = edge.weight();
                    let parent_done = engine.graph[parent_idx].status == NodeStatus::Completed;
                    let branch_match =
                        branch_label_matches(edge_data.label.as_deref(), active_branches);
                    if !(parent_done && branch_match) {
                        degree += 1;
                    }
                }
                in_degrees.insert(node_idx, degree);
                if degree == 0 {
                    ready_queue.push_back(node_idx);
                }
            }
        }
        Self {
            in_degrees,
            ready_queue,
        }
    }
}
