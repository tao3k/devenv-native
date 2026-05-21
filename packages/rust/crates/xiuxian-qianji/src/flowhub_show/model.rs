//! Flowhub show model.
//!
//! These DTOs carry root, module, and scenario-case summaries between Flowhub
//! discovery and renderer helpers without creating API facade cycles.

use std::path::PathBuf;

/// Flowhub module shape displayed by `qianji show`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowhubModuleKind {
    /// Module owns internal child-module composition.
    Composite,
    /// Module is a qianji.toml-anchored leaf node.
    Leaf,
}

/// Compact summary of one Mermaid scenario-case owned by a Flowhub node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubScenarioCaseSummary {
    /// On-disk Mermaid filename.
    pub file_name: String,
    /// Stable Mermaid graph identity resolved from `[[graph]].name` or the
    /// owning filename stem.
    pub merimind_graph_name: String,
}

/// Compact summary of one Flowhub module within a root or module render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubModuleSummary {
    /// Hierarchical module reference relative to the Flowhub root.
    pub module_ref: String,
    /// Stable module name declared by the root manifest.
    pub module_name: String,
    /// On-disk module directory.
    pub module_dir: PathBuf,
    /// Whether the module is leaf or composite.
    pub kind: FlowhubModuleKind,
    /// Stable entry export.
    pub exports_entry: String,
    /// Stable ready export.
    pub exports_ready: String,
    /// Qualified child module refs for composite modules.
    pub child_modules: Vec<String>,
    /// Immediate Mermaid scenario-case files owned by this module.
    pub scenario_cases: Vec<FlowhubScenarioCaseSummary>,
}

/// Root-level Flowhub library summary rendered by `qianji show`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubRootShow {
    /// Flowhub library root on disk.
    pub root: PathBuf,
    /// Ordered summaries of discovered modules.
    pub modules: Vec<FlowhubModuleSummary>,
}

/// Single-module Flowhub summary rendered by `qianji show`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubModuleShow {
    /// Core module summary.
    pub summary: FlowhubModuleSummary,
    /// Count of registered child graph nodes owned by this module.
    pub registered_child_count: usize,
    /// Count of required contract entries anchored by this module.
    pub required_contract_count: usize,
    /// Immediate Mermaid scenario-case files owned by this module.
    pub scenario_cases: Vec<FlowhubScenarioCaseSummary>,
}

/// First-order Flowhub display surface for either a root or one module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlowhubShow {
    /// Flowhub library root summary.
    Root(FlowhubRootShow),
    /// Single Flowhub module summary.
    Module(FlowhubModuleShow),
}
