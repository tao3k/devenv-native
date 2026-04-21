use std::path::{Path, PathBuf};

use crate::error::QianjiError;
use crate::flowhub::discover::{FlowhubDirKind, classify_flowhub_dir};
use crate::markdown::{MarkdownDiagnostic, render_validation_failed, render_validation_pass};

use super::traversal::{check_flowhub_module, check_flowhub_root};

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

/// Validate a Flowhub library root or a single Flowhub module directory.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the target is not Flowhub-shaped or
/// its filesystem cannot be traversed.
pub fn check_flowhub(dir: impl AsRef<Path>) -> Result<FlowhubCheckReport, QianjiError> {
    let dir = dir.as_ref();
    match classify_flowhub_dir(dir)? {
        Some(FlowhubDirKind::Root) => check_flowhub_root(dir),
        Some(FlowhubDirKind::Module) => check_flowhub_module(dir),
        None => Err(QianjiError::Topology(format!(
            "`{}` is not a Flowhub root or module directory",
            dir.display()
        ))),
    }
}

/// Render a Flowhub validation report into markdown diagnostics.
#[must_use]
pub fn render_flowhub_check_markdown(report: &FlowhubCheckReport) -> String {
    if report.is_valid() {
        return render_validation_pass(&[
            format!("Location: {}", report.target.display()),
            format!("Checked modules: {}", report.checked_modules),
        ]);
    }

    let diagnostics = report
        .diagnostics
        .iter()
        .map(|diagnostic| MarkdownDiagnostic {
            title: diagnostic.title.as_str(),
            location: diagnostic.location.display().to_string().into(),
            problem: diagnostic.problem.as_str(),
            why_it_blocks: diagnostic.why_it_blocks.as_str(),
            fix: diagnostic.fix.as_str(),
        })
        .collect::<Vec<_>>();

    render_validation_failed(
        &[
            format!("Location: {}", report.target.display()),
            format!("Checked modules: {}", report.checked_modules),
        ],
        &diagnostics,
    )
}
