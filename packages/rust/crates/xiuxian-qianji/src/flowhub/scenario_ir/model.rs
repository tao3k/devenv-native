use std::collections::BTreeSet;

use crate::contracts::{FlowhubGraphSurfaceContract, FlowhubGraphTopology, WorkdirCheck};
use crate::flowhub::mermaid::normalize_graph_node_label;

/// Compiled scenario-case contract consumed by `show` and Flowhub validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FlowhubScenarioIr {
    /// Stable Mermaid graph identity shown to executors.
    pub(crate) merimind_graph_name: String,
    /// Optional scenario id owned by the graph source.
    pub(crate) scenario_id: Option<String>,
    /// Optional user-facing scenario description.
    pub(crate) description: Option<String>,
    /// Optional declared topology owned by the graph source.
    pub(crate) declared_topology: Option<FlowhubGraphTopology>,
    /// Optional localized work-surface contract.
    pub(crate) workdir: Option<FlowhubScenarioWorkdirIr>,
    /// Declared node semantics keyed by Mermaid label.
    pub(crate) nodes: Vec<FlowhubScenarioNodeIr>,
}

impl FlowhubScenarioIr {
    /// Collect the allowed scenario-node labels for Mermaid validation.
    pub(crate) fn allowed_graph_node_labels(&self) -> BTreeSet<String> {
        self.nodes
            .iter()
            .map(|node| normalize_graph_node_label(node.label.as_str()))
            .collect()
    }

    /// Resolve one node contract by Mermaid label.
    pub(crate) fn node_contract(&self, label: &str) -> Option<&FlowhubScenarioNodeIr> {
        let normalized_label = normalize_graph_node_label(label);
        self.nodes
            .iter()
            .find(|node| normalize_graph_node_label(node.label.as_str()) == normalized_label)
    }
}

/// Compiled localized work-surface contract owned by one graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FlowhubScenarioWorkdirIr {
    /// Optional execution note shown before the localized surface.
    pub(crate) note: Option<String>,
    /// Symbolic run root shown in `show --graph`.
    pub(crate) root: String,
    /// Localized bounded-work checks derived into the rendered `qianji.toml`.
    pub(crate) check: WorkdirCheck,
    /// Optional persistent canonical target preview for validated merges.
    pub(crate) target: Option<FlowhubGraphSurfaceContract>,
    /// Optional declared completion gate over canonical target paths.
    pub(crate) done_gate_require: Vec<String>,
}

/// Compiled node-level contract owned by one graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FlowhubScenarioNodeIr {
    /// Exact Mermaid node label.
    pub(crate) label: String,
    /// Optional contract-owned node semantic kind.
    pub(crate) kind: Option<String>,
    /// Optional stable role description.
    pub(crate) role: Option<String>,
    /// Optional stable action guidance.
    pub(crate) agent_action: Option<String>,
    /// Optional localized checkpoint path.
    pub(crate) checkpoint: Option<String>,
    /// Optional localized writes owned by the node.
    pub(crate) writes: Vec<String>,
    /// Optional persistent merge targets owned by the node.
    pub(crate) merge_target: Vec<String>,
}
