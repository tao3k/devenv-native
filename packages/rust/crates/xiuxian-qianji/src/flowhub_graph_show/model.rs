//! Shared DTOs for `Flowhub` graph-show rendering and planning.

use std::path::PathBuf;

use crate::contracts::{FlowhubGraphNodeKind, FlowhubGraphTopology};

/// One Flowhub Mermaid graph contract preview.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubGraphShow {
    /// Mermaid graph file on disk.
    pub graph_path: PathBuf,
    /// Stable graph identity resolved from `[[graph]].name` or the filename
    /// stem fallback.
    pub merimind_graph_name: String,
    /// Optional scenario id declared in Mermaid annotations.
    pub scenario_id: Option<String>,
    /// Optional scenario description declared in Mermaid annotations.
    pub description: Option<String>,
    /// Resolved topology from petgraph analysis.
    pub topology: FlowhubGraphTopology,
    /// Optional module-owned declared topology.
    pub declared_topology: Option<FlowhubGraphTopology>,
    /// Raw Mermaid source.
    pub mermaid: String,
    /// Owning Flowhub module reference.
    pub owning_module_ref: String,
    /// Flowhub root containing the owning module.
    pub flowhub_root: PathBuf,
    /// Declared Mermaid direction such as `LR`.
    pub direction: String,
    /// Parsed nodes with semantic guidance in declaration order.
    pub nodes: Vec<FlowhubGraphNodeSummary>,
    /// Parsed edges in declaration order.
    pub edges: Vec<FlowhubGraphEdgeSummary>,
    /// Registered Flowhub modules that are missing from the Mermaid graph.
    pub missing_registered_modules: Vec<String>,
    /// Mermaid nodes outside the registered-module set and allowed graph vocabulary.
    pub unknown_graph_nodes: Vec<String>,
    /// Node labels grouped by cyclic SCC when the graph loops.
    pub cyclic_components: Vec<Vec<String>>,
    /// Static Flowhub module-owned contract entries for the graph source.
    pub module_contract_surface: Vec<String>,
    /// Declared bounded check surface for executor guidance.
    pub declared_check_surface: FlowhubGraphCheckSurface,
    /// Owning module manifest source.
    pub owning_module_manifest_toml: String,
}

/// Declared bounded check surface derived from one graph contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubGraphCheckSurface {
    /// Optional note that explains how the localized surface should be used.
    pub note: Option<String>,
    /// Optional localized run root declared by the Flowhub contract.
    pub root: Option<String>,
    /// Raw `check.require` paths or globs declared by Flowhub.
    pub required_paths: Vec<String>,
    /// Raw `check.flowchart` surfaces declared by Flowhub.
    pub flowchart_surfaces: Vec<String>,
    /// Optional persistent canonical target tree for validated merges.
    pub persistent_target_surface_tree: Vec<String>,
    /// Optional declared done-gate paths over the persistent target surface.
    pub done_gate_require: Vec<String>,
}

/// One parsed Flowhub Mermaid node summary with semantic guidance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubGraphNodeSummary {
    /// Stable Mermaid node id.
    pub id: String,
    /// Visible Mermaid label.
    pub label: String,
    /// Contract-owned node semantic kind when declared.
    pub kind: Option<FlowhubGraphNodeKind>,
    /// Stable role description for Codex.
    pub role: String,
    /// Stable agent action guidance for the node.
    pub agent_action: String,
    /// Visible next-node labels in edge order.
    pub next: Vec<String>,
    /// Resolved Flowhub module ref when the node represents a registered module.
    pub module_ref: Option<String>,
    /// Stable module entry export when available.
    pub exports_entry: Option<String>,
    /// Stable module ready export when available.
    pub exports_ready: Option<String>,
}

/// One extracted graph edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubGraphEdgeSummary {
    /// Edge source label.
    pub from_label: String,
    /// Edge destination label.
    pub to_label: String,
}
