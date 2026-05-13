//! Shared data contracts for `Flowhub` structural validation.

use std::path::PathBuf;

/// One user-facing validation diagnostic for a Flowhub root or module check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubDiagnostic {
    /// Short diagnostic title.
    pub title: String,
    /// On-disk location of the failing surface.
    pub location: PathBuf,
    /// Concrete failing condition.
    pub problem: String,
    /// Why the issue blocks continued Flowhub use.
    pub why_it_blocks: String,
    /// Concrete next action for repairing the failing surface.
    pub fix: String,
}

/// Structural validation result for a Flowhub root or single module target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubCheckReport {
    /// Checked Flowhub root or module path.
    pub target: PathBuf,
    /// Count of modules that were traversed during validation.
    pub checked_modules: usize,
    /// Collected blocking diagnostics.
    pub diagnostics: Vec<FlowhubDiagnostic>,
}

impl FlowhubCheckReport {
    /// Returns `true` when no blocking diagnostics were emitted.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }
}
