use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::analyzers::ProjectedGapKind;
use crate::analyzers::projection::ProjectionPageKind;
use crate::analyzers::{DocsPlannerItemResult, DocsPlannerQueueResult, DocsPlannerRankHit};

/// Docs-facing query for deterministic deep-wiki planner workset opening.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPlannerWorksetQuery {
    /// Repository identifier to inspect.
    pub repo_id: String,
    /// Optional projected gap kind filter.
    pub gap_kind: Option<ProjectedGapKind>,
    /// Optional projected-page family filter.
    pub page_kind: Option<ProjectionPageKind>,
    /// Maximum number of preview gaps to keep for each gap kind before batch opening.
    pub per_kind_limit: usize,
    /// Maximum number of planner items to open across the queue preview.
    pub limit: usize,
    /// Optional projected-page family to include as a deterministic cluster in each navigation bundle.
    pub family_kind: Option<ProjectionPageKind>,
    /// Maximum number of related projected pages to return for each opened planner item.
    pub related_limit: usize,
    /// Maximum number of related projected pages to return in the requested family cluster.
    pub family_limit: usize,
}

/// One deterministic grouped planner workset lane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPlannerWorksetGroup {
    /// Projected gap kind carried by this grouped workset lane.
    pub kind: ProjectedGapKind,
    /// Number of ranked gaps selected into this group.
    pub selected_count: usize,
    /// Deterministic quota hint for this grouped workset lane.
    pub quota: DocsPlannerWorksetQuotaHint,
    /// Family-aware grouped workset lanes nested inside this gap-kind group.
    pub families: Vec<DocsPlannerWorksetFamilyGroup>,
    /// Ranked hits selected for this group, preserving global rank order.
    pub ranked_hits: Vec<DocsPlannerRankHit>,
    /// Opened planner-item bundles for this group, preserving global rank order.
    pub items: Vec<DocsPlannerItemResult>,
}

/// One deterministic family-aware grouped planner workset lane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPlannerWorksetFamilyGroup {
    /// Projected page family carried by this nested workset lane.
    pub kind: ProjectionPageKind,
    /// Number of ranked gaps selected into this family group.
    pub selected_count: usize,
    /// Deterministic quota hint for this nested family group.
    pub quota: DocsPlannerWorksetQuotaHint,
    /// Ranked hits selected for this family group, preserving global rank order.
    pub ranked_hits: Vec<DocsPlannerRankHit>,
    /// Opened planner-item bundles for this family group, preserving global rank order.
    pub items: Vec<DocsPlannerItemResult>,
}

/// One deterministic distribution entry for workset balancing summaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPlannerWorksetGapKindBalanceEntry {
    /// Projected gap kind described by this entry.
    pub kind: ProjectedGapKind,
    /// Number of selected ranked gaps in this gap kind.
    pub selected_count: usize,
    /// Whether this gap-kind group stays within the deterministic quota band.
    pub within_target_band: bool,
}

/// One deterministic family distribution entry for workset balancing summaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPlannerWorksetFamilyBalanceEntry {
    /// Projected page family described by this entry.
    pub kind: ProjectionPageKind,
    /// Number of selected ranked gaps in this page family.
    pub selected_count: usize,
    /// Whether this page-family group stays within the deterministic quota band.
    pub within_target_band: bool,
}

/// Deterministic quota-band hint for one selected planner group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPlannerWorksetQuotaHint {
    /// Deterministic lower quota bound for this group.
    pub target_floor_count: usize,
    /// Deterministic upper quota bound for this group.
    pub target_ceiling_count: usize,
    /// Whether the selected count stays within the deterministic quota band.
    pub within_target_band: bool,
}

/// Machine-readable batch-strategy code for one planner workset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DocsPlannerWorksetStrategyCode {
    /// No gaps were selected into the workset.
    EmptySelection,
    /// The workset concentrates on one gap kind and one page family.
    SingleLaneFocus,
    /// The workset concentrates on one gap kind but spans multiple families.
    FamilySplitFocus,
    /// The workset spans multiple gap kinds but stays in one family.
    GapKindSplitFocus,
    /// The workset spans multiple lanes and remains balanced across them.
    BalancedMultiLane,
    /// The workset spans multiple lanes but keeps a priority-stacked shape.
    PriorityStacked,
}

/// Machine-readable reason code for one planner workset batch strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DocsPlannerWorksetStrategyReasonCode {
    /// No selected gaps exist.
    EmptySelection,
    /// Exactly one gap kind was selected.
    SingleGapKind,
    /// Multiple gap kinds were selected.
    MultipleGapKinds,
    /// Exactly one page family was selected.
    SingleFamily,
    /// Multiple page families were selected.
    MultipleFamilies,
    /// Gap-kind lanes remain within the deterministic balance band.
    GapKindBalanced,
    /// Gap-kind lanes exceed the deterministic balance band.
    GapKindStacked,
    /// Page-family lanes remain within the deterministic balance band.
    FamilyBalanced,
    /// Page-family lanes exceed the deterministic balance band.
    FamilyStacked,
}

/// One deterministic reason contributing to the planner workset batch strategy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPlannerWorksetStrategyReason {
    /// Machine-readable reason code.
    pub code: DocsPlannerWorksetStrategyReasonCode,
    /// Deterministic human-readable explanation for the strategy evidence.
    pub detail: String,
}

/// Deterministic batch-strategy summary for one planner workset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPlannerWorksetStrategy {
    /// Machine-readable workset batch-strategy code.
    pub code: DocsPlannerWorksetStrategyCode,
    /// Number of populated gap-kind groups contributing to the strategy.
    pub gap_kind_group_count: usize,
    /// Number of populated page-family groups contributing to the strategy.
    pub family_group_count: usize,
    /// Deterministic rationale for the selected batch strategy.
    pub reasons: Vec<DocsPlannerWorksetStrategyReason>,
}

/// Deterministic balancing evidence for one planner workset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPlannerWorksetBalance {
    /// Number of ranked gaps selected into this workset.
    pub selection_count: usize,
    /// Number of populated projected gap-kind groups in the selected workset.
    pub gap_kind_group_count: usize,
    /// Number of populated projected page-family groups in the selected workset.
    pub family_group_count: usize,
    /// Deterministic lower quota bound for each populated gap-kind group.
    pub gap_kind_target_floor_count: usize,
    /// Deterministic upper quota bound for each populated gap-kind group.
    pub gap_kind_target_ceiling_count: usize,
    /// Deterministic lower quota bound for each populated page-family group.
    pub family_target_floor_count: usize,
    /// Deterministic upper quota bound for each populated page-family group.
    pub family_target_ceiling_count: usize,
    /// Distribution of selected ranked gaps by projected gap kind.
    pub gap_kind_distribution: Vec<DocsPlannerWorksetGapKindBalanceEntry>,
    /// Distribution of selected ranked gaps by projected page family.
    pub family_distribution: Vec<DocsPlannerWorksetFamilyBalanceEntry>,
    /// Maximum selected-count spread across populated gap-kind groups.
    pub gap_kind_spread: usize,
    /// Maximum selected-count spread across populated page-family groups.
    pub family_spread: usize,
    /// Whether populated gap-kind groups differ by at most one selected hit.
    pub gap_kind_balanced: bool,
    /// Whether populated family groups differ by at most one selected hit.
    pub family_balanced: bool,
}

/// Docs-facing deterministic planner workset result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPlannerWorksetResult {
    /// Repository identifier inspected.
    pub repo_id: String,
    /// Deterministic queue snapshot used to choose the workset.
    pub queue: DocsPlannerQueueResult,
    /// Deterministic ranked planner gaps selected for opening.
    pub ranked_hits: Vec<DocsPlannerRankHit>,
    /// Deterministic balancing summary for the selected workset.
    pub balance: DocsPlannerWorksetBalance,
    /// Deterministic batch-strategy summary for the selected workset.
    pub strategy: DocsPlannerWorksetStrategy,
    /// Deterministic grouped workset lanes derived from the ranked selection.
    pub groups: Vec<DocsPlannerWorksetGroup>,
    /// Opened deterministic planner-item bundles selected from the ranked gaps.
    pub items: Vec<DocsPlannerItemResult>,
}
