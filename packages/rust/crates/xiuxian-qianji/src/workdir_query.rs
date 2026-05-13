//! Workdir query surface for `xiuxian-qianji`.

use std::collections::BTreeSet;
use std::path::Path;

use xiuxian_wendao_core::SqlQueryPayload;
use xiuxian_wendao_sql::bounded_work_markdown::query_bounded_work_markdown_payload;

use crate::error::QianjiError;

use super::check::{WorkdirCheckReport, WorkdirMarkdownSurface};
use super::load::load_workdir_manifest;

/// One repair-oriented SQL follow-up query derived from a failing workdir check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkdirCheckFollowUpQuery {
    /// Checked bounded workdir root.
    pub workdir: std::path::PathBuf,
    /// Markdown surfaces implied by the failing diagnostics.
    pub surfaces: Vec<WorkdirMarkdownSurface>,
    /// Default SQL query text for skeleton-oriented follow-up retrieval.
    pub query_text: String,
}

/// Execute one SQL query over the bounded-work markdown retrieval surface.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the workdir manifest cannot be
/// loaded, or when the bounded-work markdown SQL query cannot be executed.
pub async fn query_workdir_markdown_payload(
    workdir: impl AsRef<Path>,
    query_text: &str,
) -> Result<SqlQueryPayload, QianjiError> {
    let workdir = workdir.as_ref();
    let _manifest = load_workdir_manifest(workdir.join("qianji.toml"))?;
    query_bounded_work_markdown_payload(workdir, query_text)
        .await
        .map_err(QianjiError::Topology)
}

/// Derive a default bounded-work markdown follow-up query from one check report.
#[must_use]
pub fn build_workdir_check_follow_up_query(
    report: &WorkdirCheckReport,
) -> Option<WorkdirCheckFollowUpQuery> {
    if report.is_valid() {
        return None;
    }

    let mut surfaces = report
        .diagnostics
        .iter()
        .flat_map(|diagnostic| diagnostic.follow_up_surfaces.iter().copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    if surfaces.is_empty() {
        if !report.workdir.join("qianji.toml").is_file() {
            return None;
        }
        surfaces = vec![
            WorkdirMarkdownSurface::Blueprint,
            WorkdirMarkdownSurface::Plan,
        ];
    }

    Some(WorkdirCheckFollowUpQuery {
        workdir: report.workdir.clone(),
        query_text: build_default_follow_up_query_text(&surfaces),
        surfaces,
    })
}

/// Execute the default bounded markdown follow-up query for one failing workdir report.
///
/// Returns `Ok(None)` when the supplied report is already valid and no repair
/// follow-up retrieval is needed.
///
/// # Errors
///
/// Returns [`QianjiError::Topology`] when the bounded-work manifest cannot be
/// loaded or when the derived SQL query fails.
pub async fn query_workdir_check_follow_up_payload(
    report: &WorkdirCheckReport,
) -> Result<Option<SqlQueryPayload>, QianjiError> {
    let Some(query) = build_workdir_check_follow_up_query(report) else {
        return Ok(None);
    };

    query_workdir_markdown_payload(&query.workdir, &query.query_text)
        .await
        .map(Some)
}

fn build_default_follow_up_query_text(surfaces: &[WorkdirMarkdownSurface]) -> String {
    let predicate = match surfaces {
        [surface] => format!("surface = '{}'", surface.as_str()),
        _ => format!(
            "surface in ({})",
            surfaces
                .iter()
                .map(|surface| format!("'{}'", surface.as_str()))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    };

    format!(
        "select path, surface, surface_kind, heading_path, skeleton \
from markdown \
where {predicate} \
order by surface, path, heading_path"
    )
}
