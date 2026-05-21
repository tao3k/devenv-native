//! Flowhub scenario preview model.
//!
//! These DTOs are shared by the scenario preview API and renderer so renderer
//! helpers do not depend on the public API facade.

use std::path::PathBuf;

/// One visible surface preview derived from a scenario alias.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubScenarioSurfacePreview {
    /// Alias that will become a top-level bounded work-surface directory.
    pub alias: String,
    /// Resolved Flowhub module reference for this alias.
    pub module_ref: String,
    /// Conceptual target path inside the future work surface.
    pub target_path: PathBuf,
    /// Source node manifest inside Flowhub.
    pub source_manifest_path: PathBuf,
}

/// One hidden composite alias that participates in the scenario graph but does
/// not materialize into a top-level bounded surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubScenarioHiddenAlias {
    /// Alias declared by the scenario manifest.
    pub alias: String,
    /// Resolved Flowhub module reference.
    pub module_ref: String,
}

/// First-order preview of the bounded work surface implied by a scenario root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubScenarioShow {
    /// Stable scenario/plan name.
    pub plan_name: String,
    /// Scenario root directory.
    pub scenario_dir: PathBuf,
    /// Resolved Flowhub root used for module lookups.
    pub flowhub_root: PathBuf,
    /// Derived preview of the materialized root flowchart.
    pub flowchart_preview: String,
    /// Ordered visible leaf surfaces that will materialize.
    pub surfaces: Vec<FlowhubScenarioSurfacePreview>,
    /// Ordered composite aliases hidden behind the top-level bounded surface.
    pub hidden_aliases: Vec<FlowhubScenarioHiddenAlias>,
    /// Declared scenario links rendered as stable references.
    pub links: Vec<String>,
}
