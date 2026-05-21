//! `analyzers::query::gaps` owns Wendao analyzers query gaps behavior.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::analyzers::projection::ProjectionPageKind;

/// Query for deterministic projected deep-wiki gaps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedGapReportQuery {
    /// Repository identifier to inspect.
    pub repo_id: String,
}

/// Deterministic projected gap kind for deep-wiki planning.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Ord, PartialOrd,
)]
#[serde(rename_all = "snake_case")]
pub enum ProjectedGapKind {
    /// Module reference page has no documentation evidence.
    ModuleReferenceWithoutDocumentation,
    /// Symbol reference page has no documentation evidence.
    SymbolReferenceWithoutDocumentation,
    /// Symbol reference page carries unverified documentation evidence.
    SymbolReferenceUnverified,
    /// Example how-to page has no stable module or symbol anchor.
    ExampleHowToWithoutAnchor,
    /// Documentation-backed projected page has no stable module or symbol anchor.
    DocumentationPageWithoutAnchor,
}

/// One deterministic gap summary row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProjectedGapSummaryEntry {
    /// Gap kind summarized by this row.
    pub kind: ProjectedGapKind,
    /// Number of gaps of this kind.
    pub count: usize,
}

/// Deterministic projected gap summary for one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProjectedGapSummary {
    /// Number of projected pages considered during gap detection.
    pub page_count: usize,
    /// Total number of deterministic gaps.
    pub gap_count: usize,
    /// Gap counts grouped by kind.
    pub by_kind: Vec<ProjectedGapSummaryEntry>,
}

/// One deterministic deep-wiki planning gap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
/// Raw DTO boundary: this public record mirrors serialized Wendao transport fields.
pub struct ProjectedGapRecord {
    /// Repository identifier.
    pub repo_id: String,
    /// Stable gap identifier.
    pub gap_id: String,
    /// Gap kind.
    pub kind: ProjectedGapKind,
    /// Projected page family affected by the gap.
    pub page_kind: ProjectionPageKind,
    /// Stable projected page identifier carrying the gap.
    pub page_id: String,
    /// Primary repository entity identifier behind the gap.
    pub entity_id: String,
    /// Human-readable title for the affected page/entity.
    pub title: String,
    /// Primary repository-relative path for the affected page/entity.
    pub path: String,
    /// Related module identifiers.
    pub module_ids: Vec<String>,
    /// Related symbol identifiers.
    pub symbol_ids: Vec<String>,
    /// Related example identifiers.
    pub example_ids: Vec<String>,
    /// Related documentation identifiers.
    pub doc_ids: Vec<String>,
    /// Preserved format hints from the projected page.
    pub format_hints: Vec<String>,
}

/// Deterministic projected gap report for one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedGapReportResult {
    /// Repository identifier inspected.
    pub repo_id: String,
    /// Deterministic summary over the projected gap set.
    pub summary: ProjectedGapSummary,
    /// Stable gap records ordered for downstream planning.
    pub gaps: Vec<ProjectedGapRecord>,
}
