use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::analyzers::projection::ProjectionPageKind;
use crate::analyzers::{ProjectedGapKind, ProjectedGapRecord};

/// Docs-facing query for deterministic deep-wiki planner discovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPlannerSearchQuery {
    /// Repository identifier to inspect.
    pub repo_id: String,
    /// User-provided planner search string.
    pub query: String,
    /// Optional projected gap kind filter.
    pub gap_kind: Option<ProjectedGapKind>,
    /// Optional projected-page family filter.
    pub page_kind: Option<ProjectionPageKind>,
    /// Maximum number of planner hits to return.
    pub limit: usize,
}

/// One deterministic planner discovery hit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPlannerSearchHit {
    /// Stable ordering score derived from deterministic planner evidence.
    pub search_score: u8,
    /// Matching deterministic projected gap.
    pub gap: ProjectedGapRecord,
}

/// Docs-facing deterministic planner discovery result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPlannerSearchResult {
    /// Repository identifier inspected.
    pub repo_id: String,
    /// Matching deterministic planner gaps.
    pub hits: Vec<DocsPlannerSearchHit>,
}
