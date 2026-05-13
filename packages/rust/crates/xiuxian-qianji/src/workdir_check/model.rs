//! Shared data contracts for bounded workdir structural checks.

use std::path::PathBuf;

/// One bounded markdown retrieval surface supported by the compact workdir.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorkdirMarkdownSurface {
    /// The `blueprint/` markdown surface.
    Blueprint,
    /// The `plan/` markdown surface.
    Plan,
    /// The `semantic/` advisory markdown surface.
    Semantic,
}

impl WorkdirMarkdownSurface {
    /// Return the stable SQL-visible surface name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Blueprint => "blueprint",
            Self::Plan => "plan",
            Self::Semantic => "semantic",
        }
    }

    pub(super) fn from_top_level_name(surface: &str) -> Option<Self> {
        match surface {
            "blueprint" => Some(Self::Blueprint),
            "plan" => Some(Self::Plan),
            "semantic" => Some(Self::Semantic),
            _ => None,
        }
    }
}

/// One user-facing validation diagnostic for a bounded work surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkdirDiagnostic {
    /// Short diagnostic title.
    pub title: String,
    /// On-disk location of the failing surface.
    pub location: PathBuf,
    /// Concrete failing condition.
    pub problem: String,
    /// Why the issue blocks continued bounded work.
    pub why_it_blocks: String,
    /// Concrete next action for repairing the surface.
    pub fix: String,
    /// Bounded markdown surfaces that should be queried during repair follow-up.
    pub follow_up_surfaces: Vec<WorkdirMarkdownSurface>,
}

/// Structural validation result for one bounded work surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkdirCheckReport {
    /// Stable plan name from the root manifest.
    pub plan_name: String,
    /// Checked bounded workdir root.
    pub workdir: PathBuf,
    /// Collected blocking diagnostics.
    pub diagnostics: Vec<WorkdirDiagnostic>,
}

impl WorkdirCheckReport {
    /// Returns `true` when no blocking diagnostics were emitted.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }
}
