use serde::{Deserialize, Serialize};

/// Root `[plan]` table for one bounded work-surface manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkdirPlan {
    /// Stable active plan name.
    pub name: String,
    /// Top-level surfaces that `qianji show --dir` should expose.
    pub surface: Vec<String>,
}

/// Root `[check]` table for one bounded work-surface manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkdirCheck {
    /// Required exact paths or glob patterns inside the bounded work surface.
    pub require: Vec<String>,
    /// Principal surfaces that must remain visible in `flowchart.mmd`.
    pub flowchart: Vec<String>,
}

/// Compact root manifest for one bounded work surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkdirManifest {
    /// Work-surface schema version.
    pub version: u64,
    /// Active plan metadata and visible top-level surfaces.
    pub plan: WorkdirPlan,
    /// Structural and flowchart checks.
    pub check: WorkdirCheck,
}
