use std::fmt;

use merman_core::{Engine, ParseOptions, RenderSemanticModel};

use super::mermaid_model::{MermaidEdge, MermaidFlowchart, MermaidNode, MermaidNodeKind};

/// One Mermaid parse error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MermaidParseError {
    message: String,
}

impl MermaidParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for MermaidParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for MermaidParseError {}

/// Parse one Mermaid flowchart using a graph identity that has already been
/// resolved by the caller. Syntax acceptance is delegated to `merman-core`, and
/// Qianji only projects the first-order graph semantics it needs.
pub(crate) fn parse_mermaid_flowchart(
    source: &str,
    resolved_graph_name: &str,
    registered_module_names: &[String],
) -> Result<MermaidFlowchart, MermaidParseError> {
    let parsed = Engine::new()
        .parse_diagram_for_render_model_sync(source, ParseOptions::strict())
        .map_err(|error| MermaidParseError::new(error.to_string()))?
        .ok_or_else(|| MermaidParseError::new("mermaid flowchart is empty"))?;

    let RenderSemanticModel::Flowchart(flowchart) = parsed.model else {
        return Err(MermaidParseError::new(format!(
            "expected a Mermaid flowchart, but parsed `{}`",
            parsed.meta.diagram_type
        )));
    };

    let nodes = flowchart
        .nodes
        .into_iter()
        .map(|node| {
            let label = node
                .label
                .filter(|label| !label.trim().is_empty())
                .unwrap_or_else(|| node.id.clone());
            let kind = if registered_module_names
                .iter()
                .any(|module| module == &label)
            {
                MermaidNodeKind::Module
            } else {
                MermaidNodeKind::Scenario
            };

            MermaidNode {
                id: node.id,
                label,
                kind,
            }
        })
        .collect();

    let edges = flowchart
        .edges
        .into_iter()
        .map(|edge| MermaidEdge {
            from: edge.from,
            to: edge.to,
        })
        .collect();

    Ok(MermaidFlowchart {
        merimind_graph_name: resolved_graph_name.to_string(),
        direction: flowchart.direction.unwrap_or_else(|| "TB".to_string()),
        nodes,
        edges,
    })
}

#[cfg(test)]
#[path = "../../../tests/unit/flowhub/mermaid/parse.rs"]
mod tests;
