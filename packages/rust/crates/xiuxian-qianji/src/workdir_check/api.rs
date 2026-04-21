use std::path::{Path, PathBuf};

use crate::error::QianjiError;
use crate::workdir::load_workdir_manifest;

use super::filesystem::{load_optional_flowchart, validate_required_paths};
use super::flowchart::validate_flowchart_alignment;
use super::render::render_workdir_check_markdown_impl;
use super::runtime::{derive_step_aware_context, step_aware_required_paths};

/// One bounded markdown retrieval surface supported by the compact workdir.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorkdirMarkdownSurface {
    /// The `blueprint/` markdown surface.
    Blueprint,
    /// The `plan/` markdown surface.
    Plan,
}

impl WorkdirMarkdownSurface {
    /// Return the stable SQL-visible surface name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Blueprint => "blueprint",
            Self::Plan => "plan",
        }
    }

    pub(super) fn from_top_level_name(surface: &str) -> Option<Self> {
        match surface {
            "blueprint" => Some(Self::Blueprint),
            "plan" => Some(Self::Plan),
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

/// Validate the bounded work-surface contract on disk.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the root manifest cannot be loaded,
/// the filesystem cannot be inspected, or the flowchart companion cannot be
/// read.
pub fn check_workdir(workdir: impl AsRef<Path>) -> Result<WorkdirCheckReport, QianjiError> {
    let workdir = workdir.as_ref();
    let manifest = load_workdir_manifest(workdir.join("qianji.toml"))?;
    let mut diagnostics = Vec::new();
    let flowchart_path = workdir.join("flowchart.mmd");
    let flowchart = load_optional_flowchart(&flowchart_path)?;
    let step_aware = derive_step_aware_context(
        workdir,
        flowchart.as_deref(),
        &flowchart_path,
        &manifest.check.require,
        &mut diagnostics,
    )?;
    let required_paths = step_aware_required_paths(&manifest.check.require, step_aware.as_ref());
    validate_required_paths(workdir, &required_paths, &mut diagnostics)?;
    validate_flowchart_alignment(
        workdir,
        &flowchart_path,
        flowchart.as_deref(),
        &manifest.check.flowchart,
        step_aware.as_ref(),
        &mut diagnostics,
    )?;

    Ok(WorkdirCheckReport {
        plan_name: manifest.plan.name,
        workdir: workdir.to_path_buf(),
        diagnostics,
    })
}

/// Render a bounded work-surface validation report into markdown diagnostics.
#[must_use]
pub fn render_workdir_check_markdown(report: &WorkdirCheckReport) -> String {
    render_workdir_check_markdown_impl(report)
}
