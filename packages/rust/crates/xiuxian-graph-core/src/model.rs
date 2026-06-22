//! Generic graph projection model.

use std::collections::BTreeSet;
use std::fmt;

use thiserror::Error;

/// Stable node identifier used inside a graph projection.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GraphNodeId(String);

impl GraphNodeId {
    /// Create a graph node identifier.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrow the identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Consume the identifier and return the inner string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<&str> for GraphNodeId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for GraphNodeId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for GraphNodeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A node in a reusable graph projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphNode {
    id: GraphNodeId,
    label: String,
}

impl GraphNode {
    /// Create a graph node.
    #[must_use]
    pub fn new(id: impl Into<GraphNodeId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }

    /// Node identifier.
    #[must_use]
    pub fn id(&self) -> &GraphNodeId {
        &self.id
    }

    /// Human-readable node label.
    #[must_use]
    pub fn label(&self) -> &str {
        self.label.as_str()
    }
}

/// A directed edge in a reusable graph projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphEdge {
    source: GraphNodeId,
    target: GraphNodeId,
    label: Option<String>,
}

impl GraphEdge {
    /// Create an unlabeled directed graph edge.
    #[must_use]
    pub fn new(source: impl Into<GraphNodeId>, target: impl Into<GraphNodeId>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            label: None,
        }
    }

    /// Attach a human-readable edge label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Source node identifier.
    #[must_use]
    pub fn source(&self) -> &GraphNodeId {
        &self.source
    }

    /// Target node identifier.
    #[must_use]
    pub fn target(&self) -> &GraphNodeId {
        &self.target
    }

    /// Optional edge label.
    #[must_use]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
}

/// A small graph projection for relation display or graph algorithm adapters.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GraphProjection {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
}

impl GraphProjection {
    /// Create an empty graph projection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a graph projection from explicit node and edge lists.
    #[must_use]
    pub fn from_parts(nodes: Vec<GraphNode>, edges: Vec<GraphEdge>) -> Self {
        Self { nodes, edges }
    }

    /// Append a node to the projection.
    pub fn push_node(&mut self, node: GraphNode) {
        self.nodes.push(node);
    }

    /// Append an edge to the projection.
    pub fn push_edge(&mut self, edge: GraphEdge) {
        self.edges.push(edge);
    }

    /// Projection nodes in insertion order.
    #[must_use]
    pub fn nodes(&self) -> &[GraphNode] {
        self.nodes.as_slice()
    }

    /// Projection edges in insertion order.
    #[must_use]
    pub fn edges(&self) -> &[GraphEdge] {
        self.edges.as_slice()
    }

    /// Find a node by id.
    #[must_use]
    pub fn node(&self, id: &GraphNodeId) -> Option<&GraphNode> {
        self.nodes.iter().find(|node| node.id() == id)
    }

    /// Validate that ids are unique and every edge endpoint is declared.
    ///
    /// # Errors
    ///
    /// Returns an error when a node id is duplicated or an edge references a
    /// node that is not present in the projection.
    pub fn validate(&self) -> Result<(), GraphProjectionError> {
        let mut ids = BTreeSet::<&GraphNodeId>::new();
        for node in &self.nodes {
            if !ids.insert(node.id()) {
                return Err(GraphProjectionError::DuplicateNodeId {
                    id: node.id().clone(),
                });
            }
        }

        for edge in &self.edges {
            if !ids.contains(edge.source()) {
                return Err(GraphProjectionError::MissingSourceNode {
                    source_id: edge.source().clone(),
                    target_id: edge.target().clone(),
                });
            }
            if !ids.contains(edge.target()) {
                return Err(GraphProjectionError::MissingTargetNode {
                    source_id: edge.source().clone(),
                    target_id: edge.target().clone(),
                });
            }
        }

        Ok(())
    }
}

/// Validation error for a graph projection.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum GraphProjectionError {
    /// A node id appeared more than once.
    #[error("duplicate graph node id `{id}`")]
    DuplicateNodeId {
        /// Duplicated node id.
        id: GraphNodeId,
    },
    /// An edge source endpoint is not declared as a node.
    #[error("graph edge `{source_id}` -> `{target_id}` references a missing source node")]
    MissingSourceNode {
        /// Source node id.
        source_id: GraphNodeId,
        /// Target node id.
        target_id: GraphNodeId,
    },
    /// An edge target endpoint is not declared as a node.
    #[error("graph edge `{source_id}` -> `{target_id}` references a missing target node")]
    MissingTargetNode {
        /// Source node id.
        source_id: GraphNodeId,
        /// Target node id.
        target_id: GraphNodeId,
    },
}
