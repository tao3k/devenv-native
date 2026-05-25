//! Compact Mermaid graph rendering and validation.

use std::collections::BTreeSet;

use thiserror::Error;

use crate::{GraphNode, GraphNodeId, GraphProjection, GraphProjectionError};

/// Mermaid flowchart direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MermaidDirection {
    /// Left-to-right flowchart layout.
    LeftRight,
    /// Top-to-bottom flowchart layout.
    TopBottom,
}

impl MermaidDirection {
    fn token(self) -> &'static str {
        match self {
            Self::LeftRight => "LR",
            Self::TopBottom => "TB",
        }
    }
}

/// Compact Mermaid flowchart renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompactMermaidGraph {
    direction: MermaidDirection,
}

impl CompactMermaidGraph {
    /// Create a compact left-to-right Mermaid renderer.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            direction: MermaidDirection::LeftRight,
        }
    }

    /// Return a copy with a different flowchart direction.
    #[must_use]
    pub const fn with_direction(mut self, direction: MermaidDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Render a compact Mermaid flowchart.
    ///
    /// # Errors
    ///
    /// Returns an error when the graph projection is invalid, a node id is not
    /// representable as a compact Mermaid id, or `merman-core` rejects the
    /// generated diagram.
    pub fn render(self, projection: &GraphProjection) -> Result<String, MermaidGraphError> {
        projection.validate()?;
        for node in projection.nodes() {
            validate_mermaid_node_id(node.id())?;
        }

        let mut seen = BTreeSet::<GraphNodeId>::new();
        let mut diagram = format!("flowchart {}", self.direction.token());

        if projection.edges().is_empty() {
            for node in projection.nodes() {
                diagram.push(';');
                diagram.push_str(render_node_ref(node, &mut seen).as_str());
            }
        } else {
            for edge in projection.edges() {
                let source = projection.node(edge.source()).ok_or_else(|| {
                    GraphProjectionError::MissingSourceNode {
                        source_id: edge.source().to_string(),
                        target_id: edge.target().to_string(),
                    }
                })?;
                let target = projection.node(edge.target()).ok_or_else(|| {
                    GraphProjectionError::MissingTargetNode {
                        source_id: edge.source().to_string(),
                        target_id: edge.target().to_string(),
                    }
                })?;

                diagram.push(';');
                diagram.push_str(render_node_ref(source, &mut seen).as_str());
                if let Some(label) = edge.label().filter(|label| !label.trim().is_empty()) {
                    diagram.push_str("-->|");
                    diagram.push_str(escape_edge_label(label).as_str());
                    diagram.push('|');
                } else {
                    diagram.push_str("-->");
                }
                diagram.push_str(render_node_ref(target, &mut seen).as_str());
            }
        }

        validate_with_merman_core(&diagram)?;
        Ok(diagram)
    }
}

impl Default for CompactMermaidGraph {
    fn default() -> Self {
        Self::new()
    }
}

fn render_node_ref(node: &GraphNode, seen: &mut BTreeSet<GraphNodeId>) -> String {
    if seen.insert(node.id().clone()) {
        format!(
            "{}[\"{}\"]",
            node.id().as_str(),
            escape_node_label(node.label())
        )
    } else {
        node.id().to_string()
    }
}

fn validate_mermaid_node_id(id: &GraphNodeId) -> Result<(), MermaidGraphError> {
    let mut chars = id.as_str().chars();
    let Some(first) = chars.next() else {
        return Err(MermaidGraphError::InvalidNodeId { id: id.to_string() });
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return Err(MermaidGraphError::InvalidNodeId { id: id.to_string() });
    }
    if chars.any(|character| !(character == '_' || character.is_ascii_alphanumeric())) {
        return Err(MermaidGraphError::InvalidNodeId { id: id.to_string() });
    }
    Ok(())
}

fn validate_with_merman_core(diagram: &str) -> Result<(), MermaidGraphError> {
    let parsed = merman_core::Engine::new()
        .parse_diagram_sync(diagram, merman_core::ParseOptions::strict())
        .map_err(|error| MermaidGraphError::Parse {
            message: error.to_string(),
        })?;
    if parsed.is_none() {
        return Err(MermaidGraphError::Undetected);
    }
    Ok(())
}

fn escape_node_label(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn escape_edge_label(value: &str) -> String {
    value.replace('|', "\\|")
}

/// Compact Mermaid rendering error.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum MermaidGraphError {
    /// The input projection is invalid.
    #[error(transparent)]
    Projection(#[from] GraphProjectionError),
    /// A graph node id cannot be used as a compact Mermaid id.
    #[error("graph node id `{id}` is not a compact Mermaid identifier")]
    InvalidNodeId {
        /// Invalid node id.
        id: String,
    },
    /// `merman-core` did not detect the generated diagram.
    #[error("generated Mermaid graph was not detected by merman-core")]
    Undetected,
    /// `merman-core` rejected the generated diagram.
    #[error("generated Mermaid graph failed parser validation: {message}")]
    Parse {
        /// Parser error message.
        message: String,
    },
}
