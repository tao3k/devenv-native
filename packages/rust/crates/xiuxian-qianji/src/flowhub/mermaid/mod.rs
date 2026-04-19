//! Mermaid parsing and validation for Flowhub scenario-case graphs.

mod model;
mod parse;
mod topology;
mod validate;

pub(crate) use model::{MermaidEdge, MermaidFlowchart, MermaidNodeKind};
pub(crate) use parse::parse_mermaid_flowchart;
pub(crate) use topology::analyze_mermaid_flowchart_topology;
pub(crate) use validate::{
    normalize_graph_node_label, scenario_graph_label_is_allowed, validate_mermaid_flowchart,
};
