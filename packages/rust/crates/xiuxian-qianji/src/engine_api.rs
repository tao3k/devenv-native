//! Engine api surface for `xiuxian-qianji`.

use crate::consensus::ConsensusPolicy;
use crate::contracts::{NodeStatus, QianjiMechanism};
use petgraph::Directed;
use petgraph::stable_graph::{NodeIndex, StableGraph};
use std::sync::Arc;

/// Execution affinity hints for routing nodes to specific agents or roles.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NodeExecutionAffinity {
    /// Optional agent id requirement.
    pub agent_id: Option<String>,
    /// Optional role class requirement.
    pub role_class: Option<String>,
}

/// Represents a single thought mechanism node in the execution graph.
#[derive(Clone)]
pub struct QianjiNode {
    /// Unique ID of the node.
    pub id: String,
    /// Current execution status.
    pub status: NodeStatus,
    /// The logic to be executed.
    pub mechanism: Arc<dyn QianjiMechanism>,
    /// Optional consensus policy for multi-agent voting.
    pub consensus: Option<ConsensusPolicy>,
    /// Execution affinity hints for this node.
    pub execution_affinity: NodeExecutionAffinity,
}

/// Represents an edge between nodes with optional label and weight.
#[derive(Debug, Clone)]
pub struct QianjiEdge {
    /// Label for branch selection.
    pub label: Option<String>,
    /// Probability or priority weight.
    pub weight: f32,
}

/// The stateful execution engine holding the graph structure.
#[derive(Clone)]
pub struct QianjiEngine {
    /// The underlying petgraph structure.
    pub graph: StableGraph<QianjiNode, QianjiEdge, Directed>,
}

impl QianjiEngine {
    /// Creates an empty engine.
    #[must_use]
    pub fn new() -> Self {
        Self {
            graph: StableGraph::new(),
        }
    }

    /// Adds a mechanism to the graph.
    /// Identifier boundary: this public compatibility seam accepts externally owned ids.
    pub fn add_mechanism(&mut self, id: &str, mechanism: Arc<dyn QianjiMechanism>) -> NodeIndex {
        self.add_mechanism_with_affinity(id, mechanism, None, NodeExecutionAffinity::default())
    }

    /// Adds a mechanism with consensus and execution affinity metadata.
    /// Identifier boundary: this public compatibility seam accepts externally owned ids.
    pub fn add_mechanism_with_affinity(
        &mut self,
        id: &str,
        mechanism: Arc<dyn QianjiMechanism>,
        consensus: Option<ConsensusPolicy>,
        execution_affinity: NodeExecutionAffinity,
    ) -> NodeIndex {
        self.graph.add_node(QianjiNode {
            id: id.to_string(),
            status: NodeStatus::Idle,
            mechanism,
            consensus,
            execution_affinity,
        })
    }

    /// Adds a directional link between mechanisms.
    pub fn add_link(&mut self, from: NodeIndex, to: NodeIndex, label: Option<&str>, weight: f32) {
        self.graph.add_edge(
            from,
            to,
            QianjiEdge {
                label: label.map(ToString::to_string),
                weight,
            },
        );
    }
}

impl Default for QianjiEngine {
    fn default() -> Self {
        Self::new()
    }
}
