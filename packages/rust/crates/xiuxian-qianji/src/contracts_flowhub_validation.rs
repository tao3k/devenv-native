use serde::{Deserialize, Serialize};

/// Validation scope for Flowhub manifests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowhubValidationScope {
    /// Validate the module's own on-disk structure.
    Module,
    /// Validate the module after it is materialized into a scenario alias.
    Scenario,
}

/// Validation target kind for Flowhub manifests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowhubValidationKind {
    /// Directory requirement.
    Dir,
    /// File requirement.
    File,
    /// Glob requirement.
    Glob,
}

/// One validation rule declared by a module-root Flowhub manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowhubValidationRule {
    /// Validation application scope.
    pub scope: FlowhubValidationScope,
    /// Relative path or glob pattern to inspect.
    pub path: String,
    /// Expected filesystem target kind.
    pub kind: FlowhubValidationKind,
    /// Whether the path must exist.
    #[serde(default)]
    pub required: bool,
    /// Minimum required matches for glob-based validation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_matches: Option<usize>,
}
