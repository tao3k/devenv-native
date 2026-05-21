//! Public entry points for bounded workdir structural checks.

use std::path::Path;

use crate::error::QianjiError;
use crate::workdir::load_workdir_manifest;

use super::filesystem::{load_optional_flowchart, validate_required_paths};
use super::flowchart::validate_flowchart_alignment;
use super::model::WorkdirCheckReport;
use super::render::render_workdir_check_markdown_impl;
use super::runtime::{derive_step_aware_context, step_aware_required_paths};

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
