use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::contracts::WorkdirCheck;

use super::flowhub_contract::FlowhubStructureContract;
use super::flowhub_grammar::{TemplateLinkSpec, TemplateUseSpec};
use super::flowhub_validation::FlowhubValidationRule;

/// Supported Flowhub scenario-case topology classifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowhubGraphTopology {
    /// A graph with no directed cycles.
    Dag,
    /// A graph with at least one cycle and at least one acyclic exit path.
    BoundedLoop,
    /// A graph with at least one cycle but no acyclic exit path.
    OpenLoop,
}

impl FlowhubGraphTopology {
    /// Return the stable manifest/display spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dag => "dag",
            Self::BoundedLoop => "bounded_loop",
            Self::OpenLoop => "open_loop",
        }
    }
}

/// One graph-node semantic contract owned by a Mermaid scenario-case contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowhubGraphNodeContract {
    /// Exact Mermaid node label owned by this graph contract.
    pub label: String,
    /// Contract-owned node semantic kind.
    pub kind: String,
    /// Stable role description shown on `qianji show --graph`.
    pub role: String,
    /// Stable action guidance shown on `qianji show --graph`.
    pub agent_action: String,
}

/// One rendered surface preview rooted at one symbolic workdir or target root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowhubGraphSurfaceContract {
    /// Symbolic root shown for the rendered tree.
    pub root: String,
    /// Relative paths rooted under `root`.
    #[serde(default)]
    pub paths: Vec<String>,
}

/// One localized work-surface contract owned by a Mermaid scenario-case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowhubGraphWorkdirContract {
    /// Optional execution note shown before the localized work surface.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Symbolic run root shown for the localized work surface.
    pub root: String,
    /// Localized bounded-work checks derived into the rendered `qianji.toml`.
    pub check: WorkdirCheck,
    /// Optional persistent canonical target preview for validated merges.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<FlowhubGraphSurfaceContract>,
}

/// One immediate Mermaid scenario-case contract owned by a Flowhub module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowhubGraphContract {
    /// Immediate `.mmd` file owned by the module.
    pub path: String,
    /// Optional stable graph identity when the filename stem is not the desired
    /// LLM-facing graph name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Declared topology classification for the graph.
    pub topology: FlowhubGraphTopology,
    /// Optional localized work-surface contract for `qianji show --graph`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workdir: Option<FlowhubGraphWorkdirContract>,
    /// Declared graph-node semantics owned by this graph contract.
    #[serde(default)]
    pub node: Vec<FlowhubGraphNodeContract>,
}

impl FlowhubGraphContract {
    /// Resolve the graph name declared by contract, falling back to the owning
    /// filename stem when no explicit name is present.
    #[must_use]
    pub fn resolved_name_or<'a>(&'a self, fallback: &'a str) -> &'a str {
        self.name.as_deref().unwrap_or(fallback)
    }

    /// Resolve the localized bounded-work manifest name from the graph path
    /// stem, falling back to the optional declared graph alias only if the
    /// path cannot provide one.
    #[must_use]
    pub fn resolved_workdir_name(&self) -> Option<&str> {
        Path::new(&self.path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .or(self.name.as_deref())
    }
}

/// Flowhub module metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowhubModuleMetadata {
    /// Stable module name within one Flowhub.
    pub name: String,
    /// Discovery and query metadata.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Stable exported handles declared by a Flowhub module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowhubModuleExports {
    /// Primary entry handle.
    pub entry: String,
    /// Primary completion handle.
    pub ready: String,
}

/// Shared `[template]` composition table used by scenario roots and composite
/// modules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowhubTemplateComposition {
    /// Selected Flowhub modules with explicit aliases.
    #[serde(rename = "use")]
    pub use_entries: Vec<TemplateUseSpec>,
    /// Optional graph links between selected modules or local symbols.
    #[serde(default)]
    pub link: Vec<TemplateLinkSpec>,
}

/// Module-root Flowhub manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowhubModuleManifest {
    /// Planning manifest schema version.
    pub version: u64,
    /// Module metadata.
    pub module: FlowhubModuleMetadata,
    /// Stable exported handles.
    pub exports: FlowhubModuleExports,
    /// Optional child-node filesystem contract for owned subgraphs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract: Option<FlowhubStructureContract>,
    /// Optional immediate Mermaid scenario-case graph contracts.
    #[serde(default)]
    pub graph: Vec<FlowhubGraphContract>,
    /// Optional internal child-module composition for composite modules.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<FlowhubTemplateComposition>,
    /// Declared validation rules.
    #[serde(default)]
    pub validation: Vec<FlowhubValidationRule>,
}

/// Scenario metadata for Flowhub composition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowhubScenarioPlanning {
    /// Stable scenario name.
    pub name: String,
    /// Scenario tags for discovery and diagnostics.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Backward-compatible alias for the scenario-root `[template]` table.
pub type FlowhubScenarioTemplate = FlowhubTemplateComposition;

/// Scenario-root Flowhub manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowhubScenarioManifest {
    /// Planning manifest schema version.
    pub version: u64,
    /// Scenario metadata.
    pub planning: FlowhubScenarioPlanning,
    /// Selected modules and link declarations.
    pub template: FlowhubTemplateComposition,
}
