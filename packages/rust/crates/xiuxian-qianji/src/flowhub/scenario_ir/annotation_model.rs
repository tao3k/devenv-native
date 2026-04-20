use std::collections::BTreeMap;

use crate::contracts::FlowhubGraphTopology;

/// Parsed `%% qianji.*` annotations from one Mermaid scenario-case source.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct FlowhubGraphAnnotations {
    /// Scenario-level metadata.
    pub(crate) scenario: FlowhubGraphScenarioAnnotations,
    /// Node-level metadata keyed by the annotation node reference.
    pub(crate) nodes: BTreeMap<String, FlowhubGraphNodeAnnotations>,
    /// Canonical completion requirements for the done gate.
    pub(crate) done_gate_require: Vec<String>,
}

/// Scenario-level metadata owned by one graph.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct FlowhubGraphScenarioAnnotations {
    pub(crate) id: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) note: Option<String>,
    pub(crate) topology: Option<FlowhubGraphTopology>,
    pub(crate) workdir_root: Option<String>,
    pub(crate) requires: Vec<String>,
    pub(crate) target_root: Option<String>,
    pub(crate) target_paths: Vec<String>,
}

/// Node-level metadata owned by one graph.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct FlowhubGraphNodeAnnotations {
    pub(crate) kind: Option<String>,
    pub(crate) role: Option<String>,
    pub(crate) agent_action: Option<String>,
    pub(crate) checkpoint: Option<String>,
    pub(crate) writes: Vec<String>,
    pub(crate) merge_target: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum AnnotationValue {
    Scalar(String),
    List(Vec<String>),
}
