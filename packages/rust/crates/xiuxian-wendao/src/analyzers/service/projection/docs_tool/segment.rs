//! `analyzers::service::projection::docs_tool::segment` owns Wendao projection docs tool segment behavior.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::projection::{ProjectedMarkdownDocument, ProjectionPageKind};

/// Precise docs-facing projected markdown segment reopened by stable page id
/// and 1-based inclusive line range.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
/// Raw DTO boundary: this public record mirrors serialized Wendao transport fields.
pub struct DocsDocumentSegmentResult {
    /// Repository identifier projected.
    pub repo_id: String,
    /// Stable projected page identifier.
    pub page_id: String,
    /// Projected page kind.
    pub kind: ProjectionPageKind,
    /// Virtual projected markdown path.
    pub path: String,
    /// Page title.
    pub title: String,
    /// Effective 1-based inclusive line range returned to the caller.
    pub line_range: (usize, usize),
    /// Total logical line count in the projected markdown document.
    pub line_count: usize,
    /// Reopened projected markdown content for the requested line range.
    pub content: String,
}

/// Build one docs-facing projected markdown segment from a rendered projected
/// markdown document.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::AnalysisFailed`] when the requested line
/// range is invalid for the rendered markdown document.
pub(crate) fn build_document_segment(
    document: &ProjectedMarkdownDocument,
    line_start: usize,
    line_end: usize,
) -> Result<DocsDocumentSegmentResult, RepoIntelligenceError> {
    if line_start == 0 || line_end == 0 {
        return Err(RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "document segment line ranges are 1-based; received {}-{} for `{}`",
                line_start, line_end, document.page_id
            ),
        });
    }
    if line_start > line_end {
        return Err(RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "document segment line_start must be <= line_end; received {}-{} for `{}`",
                line_start, line_end, document.page_id
            ),
        });
    }

    let lines = document.markdown.lines().collect::<Vec<_>>();
    if lines.is_empty() {
        return Err(RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "projected markdown for `{}` is empty and cannot provide line segments",
                document.page_id
            ),
        });
    }
    if line_start > lines.len() {
        return Err(RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "document segment line_start {} exceeds line_count {} for `{}`",
                line_start,
                lines.len(),
                document.page_id
            ),
        });
    }

    let effective_end = line_end.min(lines.len());
    let content = lines[(line_start - 1)..effective_end].join("\n");

    Ok(DocsDocumentSegmentResult {
        repo_id: document.repo_id.clone(),
        page_id: document.page_id.clone(),
        kind: document.kind,
        path: document.path.clone(),
        title: document.title.clone(),
        line_range: (line_start, effective_end),
        line_count: lines.len(),
        content,
    })
}

#[cfg(test)]
#[path = "../../../../../tests/unit/analyzers/service/projection/docs_tool/segment.rs"]
mod tests;
