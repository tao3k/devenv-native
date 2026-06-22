//! `WendaoGraph` evidence request table columns.

use super::types::{WendaoGraphEvidenceColumnContract, WendaoGraphEvidenceColumnType, column};

pub(super) const NODE_COLUMNS: [WendaoGraphEvidenceColumnContract; 1] =
    [column("node_id", WendaoGraphEvidenceColumnType::Utf8)];
pub(super) const EDGE_COLUMNS: [WendaoGraphEvidenceColumnContract; 2] = [
    column("source_id", WendaoGraphEvidenceColumnType::Utf8),
    column("target_id", WendaoGraphEvidenceColumnType::Utf8),
];
pub(super) const SEED_COLUMNS: [WendaoGraphEvidenceColumnContract; 2] = [
    column("node_id", WendaoGraphEvidenceColumnType::Utf8),
    column("weight", WendaoGraphEvidenceColumnType::Float64),
];
pub(super) const SEMANTIC_NEIGHBOR_COLUMNS: [WendaoGraphEvidenceColumnContract; 6] = [
    column("query_id", WendaoGraphEvidenceColumnType::Utf8),
    column("neighbor_id", WendaoGraphEvidenceColumnType::Utf8),
    column("query_index", WendaoGraphEvidenceColumnType::Int64),
    column("neighbor_index", WendaoGraphEvidenceColumnType::Int64),
    column("rank", WendaoGraphEvidenceColumnType::Int64),
    column("distance", WendaoGraphEvidenceColumnType::Float64),
];
pub(super) const SEMANTIC_OVERLAY_COLUMNS: [WendaoGraphEvidenceColumnContract; 8] = [
    column("source_id", WendaoGraphEvidenceColumnType::Utf8),
    column("target_id", WendaoGraphEvidenceColumnType::Utf8),
    column("source_index", WendaoGraphEvidenceColumnType::Int64),
    column("target_index", WendaoGraphEvidenceColumnType::Int64),
    column("rank", WendaoGraphEvidenceColumnType::Int64),
    column("distance", WendaoGraphEvidenceColumnType::Float64),
    column("weight", WendaoGraphEvidenceColumnType::Float64),
    column("edge_kind", WendaoGraphEvidenceColumnType::Utf8),
];
