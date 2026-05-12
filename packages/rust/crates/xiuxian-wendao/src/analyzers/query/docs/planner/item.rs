//! `analyzers::query::docs::planner::item` owns Wendao docs planner item behavior.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::analyzers::projection::ProjectionPageKind;
use crate::analyzers::{
    ProjectedGapRecord, ProjectedRetrievalHit, RepoProjectedPageNavigationResult,
};

/// Docs-facing query for one deterministic deep-wiki planner item opened by stable gap
/// identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPlannerItemQuery {
    /// Repository identifier to inspect.
    pub repo_id: String,
    /// Stable projected gap identifier.
    pub gap_id: String,
    /// Optional projected-page family to include as a deterministic cluster.
    pub family_kind: Option<ProjectionPageKind>,
    /// Maximum number of related projected pages to return.
    pub related_limit: usize,
    /// Maximum number of related projected pages to return in the requested family cluster.
    pub family_limit: usize,
}

/// Docs-facing deterministic planner item bundle for one stable projected gap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPlannerItemResult {
    /// Repository identifier inspected.
    pub repo_id: String,
    /// The requested deterministic projected gap.
    pub gap: ProjectedGapRecord,
    /// Deterministic mixed retrieval hit for the gap page.
    pub hit: ProjectedRetrievalHit,
    /// Deterministic navigation bundle for the gap page.
    pub navigation: RepoProjectedPageNavigationResult,
}
