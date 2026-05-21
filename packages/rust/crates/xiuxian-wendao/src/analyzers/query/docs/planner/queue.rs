//! `analyzers::query::docs::planner::queue` owns Wendao docs planner queue behavior.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::analyzers::projection::ProjectionPageKind;
use crate::analyzers::{ProjectedGapKind, ProjectedGapRecord};

/// Docs-facing query for deterministic deep-wiki planner queue shaping.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPlannerQueueQuery {
    /// Repository identifier to inspect.
    pub repo_id: String,
    /// Optional projected gap kind filter.
    pub gap_kind: Option<ProjectedGapKind>,
    /// Optional projected-page family filter.
    pub page_kind: Option<ProjectionPageKind>,
    /// Maximum number of preview gaps to return for each gap kind.
    pub per_kind_limit: usize,
}

/// One grouped deterministic planner queue lane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPlannerQueueGroup {
    /// Projected gap kind carried by this queue group.
    pub kind: ProjectedGapKind,
    /// Total number of matching gaps in this group before preview truncation.
    pub count: usize,
    /// Deterministic preview of matching gaps in this group.
    pub gaps: Vec<ProjectedGapRecord>,
}

/// Docs-facing deterministic planner queue result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPlannerQueueResult {
    /// Repository identifier inspected.
    pub repo_id: String,
    /// Number of projected pages considered in the underlying gap report.
    pub page_count: usize,
    /// Number of matching gaps across all queue groups.
    pub total_gap_count: usize,
    /// Deterministic gap groups for planner queue shaping.
    pub groups: Vec<DocsPlannerQueueGroup>,
}
