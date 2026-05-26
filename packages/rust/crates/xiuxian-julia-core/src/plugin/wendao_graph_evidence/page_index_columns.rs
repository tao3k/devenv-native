//! `WendaoGraph` `PageIndex` reasoning table columns.

use super::types::{WendaoGraphEvidenceColumnContract, WendaoGraphEvidenceColumnType, column};

pub(super) const PAGE_INDEX_REASONING_NODE_COLUMNS: [WendaoGraphEvidenceColumnContract; 10] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("page_id", WendaoGraphEvidenceColumnType::Utf8),
    column("parent_id", WendaoGraphEvidenceColumnType::Utf8),
    column("depth", WendaoGraphEvidenceColumnType::Int64),
    column("rank", WendaoGraphEvidenceColumnType::Int64),
    column("title", WendaoGraphEvidenceColumnType::Utf8),
    column("summary", WendaoGraphEvidenceColumnType::Utf8),
    column("line_start", WendaoGraphEvidenceColumnType::Int64),
    column("line_end", WendaoGraphEvidenceColumnType::Int64),
    column("token_count", WendaoGraphEvidenceColumnType::Int64),
];
pub(super) const PAGE_INDEX_REASONING_EDGE_COLUMNS: [WendaoGraphEvidenceColumnContract; 4] = [
    column("source_id", WendaoGraphEvidenceColumnType::Utf8),
    column("target_id", WendaoGraphEvidenceColumnType::Utf8),
    column("edge_kind", WendaoGraphEvidenceColumnType::Utf8),
    column("weight", WendaoGraphEvidenceColumnType::Float64),
];
pub(super) const PAGE_INDEX_REASONING_SEED_COLUMNS: [WendaoGraphEvidenceColumnContract; 3] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("weight", WendaoGraphEvidenceColumnType::Float64),
    column("seed_kind", WendaoGraphEvidenceColumnType::Utf8),
];
pub(super) const PAGE_INDEX_REASONING_FRONTIER_COLUMNS: [WendaoGraphEvidenceColumnContract; 10] = [
    column("tree_id", WendaoGraphEvidenceColumnType::Utf8),
    column("parent_step_id", WendaoGraphEvidenceColumnType::Utf8),
    column("step_id", WendaoGraphEvidenceColumnType::Utf8),
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("page_id", WendaoGraphEvidenceColumnType::Utf8),
    column("depth", WendaoGraphEvidenceColumnType::Int64),
    column("rank", WendaoGraphEvidenceColumnType::Int64),
    column("score", WendaoGraphEvidenceColumnType::Float64),
    column("decision_kind", WendaoGraphEvidenceColumnType::Utf8),
    column("disclosure_budget", WendaoGraphEvidenceColumnType::Int64),
];
pub(super) const PAGE_INDEX_DISCLOSURE_TRACE_COLUMNS: [WendaoGraphEvidenceColumnContract; 8] = [
    column("tree_id", WendaoGraphEvidenceColumnType::Utf8),
    column("step_id", WendaoGraphEvidenceColumnType::Utf8),
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("page_id", WendaoGraphEvidenceColumnType::Utf8),
    column("line_start", WendaoGraphEvidenceColumnType::Int64),
    column("line_end", WendaoGraphEvidenceColumnType::Int64),
    column("evidence_kind", WendaoGraphEvidenceColumnType::Utf8),
    column("reason", WendaoGraphEvidenceColumnType::Utf8),
];
pub(super) const PAGE_INDEX_PLANNER_ACTION_COLUMNS: [WendaoGraphEvidenceColumnContract; 8] = [
    column("tree_id", WendaoGraphEvidenceColumnType::Utf8),
    column("action_id", WendaoGraphEvidenceColumnType::Utf8),
    column("source_step_id", WendaoGraphEvidenceColumnType::Utf8),
    column("action_kind", WendaoGraphEvidenceColumnType::Utf8),
    column("target_step_id", WendaoGraphEvidenceColumnType::Utf8),
    column("target_node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("score", WendaoGraphEvidenceColumnType::Float64),
    column("reason", WendaoGraphEvidenceColumnType::Utf8),
];
