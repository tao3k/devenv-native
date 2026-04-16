use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::analyzers::DocRecord;
use crate::analyzers::RepoProjectedGapReportResult;

/// Query for documentation coverage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocCoverageQuery {
    /// Repository identifier to search within.
    pub repo_id: String,
    /// Optional module identifier scope.
    pub module_id: Option<String>,
}

/// Minimal documentation coverage response for the MVP surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocCoverageResult {
    /// Repository identifier searched.
    pub repo_id: String,
    /// Optional module identifier scope.
    pub module_id: Option<String>,
    /// Documentation rows relevant to the requested scope.
    pub docs: Vec<DocRecord>,
    /// Count of covered symbols in scope.
    pub covered_symbols: usize,
    /// Count of uncovered symbols in scope.
    pub uncovered_symbols: usize,
    /// Optional repository-level hierarchical URI for path mapping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hierarchical_uri: Option<String>,
    /// Optional repository hierarchy segments for breadcrumbs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hierarchy: Option<Vec<String>>,
}

/// Docs-facing query for deterministic projected deep-wiki gaps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsProjectedGapReportQuery {
    /// Repository identifier to inspect.
    pub repo_id: String,
}

/// Docs-facing deterministic projected deep-wiki gap report.
pub type DocsProjectedGapReportResult = RepoProjectedGapReportResult;
