use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::analyzers::projection::ProjectionPageKind;
use crate::analyzers::{ProjectedGapKind, ProjectedGapRecord};

/// Docs-facing query for deterministic deep-wiki planner ranking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPlannerRankQuery {
    /// Repository identifier to inspect.
    pub repo_id: String,
    /// Optional projected gap kind filter.
    pub gap_kind: Option<ProjectedGapKind>,
    /// Optional projected-page family filter.
    pub page_kind: Option<ProjectionPageKind>,
    /// Maximum number of ranked planner gaps to return.
    pub limit: usize,
}

/// Machine-readable deterministic priority reason code for one ranked planner gap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DocsPlannerRankReasonCode {
    /// Base score derived from the projected gap kind.
    GapKindBase,
    /// Bonus applied when the gap page is a `Reference` page.
    ReferencePageBonus,
    /// Bonus applied when the gap page is an `Explanation` page.
    ExplanationPageBonus,
    /// Bonus derived from attached module anchors.
    ModuleAnchorBonus,
    /// Bonus derived from attached symbol anchors.
    SymbolAnchorBonus,
    /// Bonus derived from attached example anchors.
    ExampleAnchorBonus,
    /// Bonus derived from attached documentation anchors.
    DocAnchorBonus,
}

/// One deterministic priority reason for a ranked planner gap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPlannerRankReason {
    /// Machine-readable reason code.
    pub code: DocsPlannerRankReasonCode,
    /// Number of priority points contributed by this reason.
    pub points: u8,
    /// Deterministic human-readable explanation for the contribution.
    pub detail: String,
}

/// One deterministic planner ranking hit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPlannerRankHit {
    /// Stable deterministic planner priority score.
    pub priority_score: u8,
    /// Deterministic explanation of the score composition.
    pub reasons: Vec<DocsPlannerRankReason>,
    /// Matching deterministic projected gap.
    pub gap: ProjectedGapRecord,
}

/// Docs-facing deterministic planner ranking result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPlannerRankResult {
    /// Repository identifier inspected.
    pub repo_id: String,
    /// Ranked deterministic planner gaps.
    pub hits: Vec<DocsPlannerRankHit>,
}
