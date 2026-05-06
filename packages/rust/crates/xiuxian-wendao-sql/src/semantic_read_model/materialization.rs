use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{
    SemanticReadModelSnapshot, semantic_read_model_snapshot_check,
    semantic_read_model_snapshot_from_root,
};

/// Target engine label for the first semantic read-model materialization plan.
pub const SEMANTIC_READ_MODEL_MATERIALIZATION_TARGET_ENGINE: &str = "duckdb";

/// Required refresh discipline for future physical semantic read-model swaps.
pub const SEMANTIC_READ_MODEL_MATERIALIZATION_REFRESH_DISCIPLINE: &str = "snapshot_swap";

/// Planned registration strategy for future `DuckDB` staging tables.
pub const SEMANTIC_READ_MODEL_PLANNED_REGISTRATION_STRATEGY: &str =
    "duckdb_materialized_arrow_staging";

/// Planned materialization state for future `DuckDB` staging tables.
pub const SEMANTIC_READ_MODEL_PLANNED_MATERIALIZATION_STATE: &str = "materialized";

/// Policy marker that prevents read-model refreshes from becoming authority writes.
pub const SEMANTIC_READ_MODEL_WRITEBACK_POLICY: &str = "read_model_only_no_semantic_writeback";

/// Read-only plan for future semantic read-model materialization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticReadModelMaterializationPlan {
    /// Whether the plan describes advisory derived rows.
    pub advisory: bool,
    /// Canonical authority that owns the source facts.
    pub authority: String,
    /// Planned downstream read-model engine.
    pub target_engine: String,
    /// Whether the plan is ready or blocked by its expected-snapshot gate.
    pub status: SemanticReadModelMaterializationStatus,
    /// Current aggregate snapshot revision.
    pub snapshot_revision: String,
    /// Optional operator-provided expected aggregate snapshot revision.
    pub expected_snapshot_revision: Option<String>,
    /// Expected-snapshot match result when an expected revision is provided.
    pub snapshot_matches_expected: Option<bool>,
    /// Required refresh discipline for future physical materialization.
    pub refresh_discipline: String,
    /// Writeback policy for future materialization.
    pub writeback_policy: String,
    /// Per-table materialization plan.
    pub tables: Vec<SemanticReadModelMaterializationTablePlan>,
    /// Ordered high-level steps required for a physical materialization pass.
    pub required_steps: Vec<String>,
}

/// Materialization plan status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticReadModelMaterializationStatus {
    /// Plan can proceed with the current snapshot.
    Ready,
    /// Plan is blocked by a failed expected-snapshot gate.
    Blocked,
}

impl SemanticReadModelMaterializationStatus {
    /// Stable human and JSON label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Blocked => "blocked",
        }
    }
}

/// Read-only materialization plan for one semantic read-model table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticReadModelMaterializationTablePlan {
    /// Table name exposed to read-only query consumers.
    pub name: String,
    /// Current projected row count.
    pub row_count: usize,
    /// Number of exposed columns.
    pub column_count: usize,
    /// Deterministic row revision for the table.
    pub row_revision: String,
    /// Planned downstream registration strategy.
    pub planned_registration_strategy: String,
    /// Planned materialization state.
    pub planned_materialization_state: String,
}

/// Build a read-only semantic read-model materialization plan from a semantic artifact root.
///
/// # Errors
///
/// Returns an error when the semantic repository under `root` is invalid, row
/// JSON metadata cannot be encoded, or `expected_snapshot_revision` is present
/// but not a valid `blake3:` revision.
pub fn semantic_read_model_materialization_plan_from_root(
    root: impl AsRef<Path>,
    expected_snapshot_revision: Option<&str>,
) -> Result<SemanticReadModelMaterializationPlan, String> {
    let snapshot = semantic_read_model_snapshot_from_root(root)?;
    semantic_read_model_materialization_plan(snapshot, expected_snapshot_revision)
}

/// Build a read-only materialization plan from one semantic read-model snapshot.
///
/// # Errors
///
/// Returns an error when `expected_snapshot_revision` is present but not a
/// valid `blake3:` revision.
pub fn semantic_read_model_materialization_plan(
    snapshot: SemanticReadModelSnapshot,
    expected_snapshot_revision: Option<&str>,
) -> Result<SemanticReadModelMaterializationPlan, String> {
    let snapshot_check = expected_snapshot_revision
        .map(|revision| semantic_read_model_snapshot_check(snapshot.clone(), revision))
        .transpose()?;
    let snapshot_matches_expected = snapshot_check.as_ref().map(|check| check.matches);
    let status = if snapshot_matches_expected.is_some_and(|matches| !matches) {
        SemanticReadModelMaterializationStatus::Blocked
    } else {
        SemanticReadModelMaterializationStatus::Ready
    };
    let tables = snapshot
        .tables
        .iter()
        .map(|table| SemanticReadModelMaterializationTablePlan {
            name: table.name.clone(),
            row_count: table.row_count,
            column_count: table.column_count,
            row_revision: table.row_revision.clone(),
            planned_registration_strategy: SEMANTIC_READ_MODEL_PLANNED_REGISTRATION_STRATEGY
                .to_string(),
            planned_materialization_state: SEMANTIC_READ_MODEL_PLANNED_MATERIALIZATION_STATE
                .to_string(),
        })
        .collect::<Vec<_>>();
    Ok(SemanticReadModelMaterializationPlan {
        advisory: true,
        authority: snapshot.authority,
        target_engine: SEMANTIC_READ_MODEL_MATERIALIZATION_TARGET_ENGINE.to_string(),
        status,
        snapshot_revision: snapshot.snapshot_revision,
        expected_snapshot_revision: expected_snapshot_revision.map(str::to_string),
        snapshot_matches_expected,
        refresh_discipline: SEMANTIC_READ_MODEL_MATERIALIZATION_REFRESH_DISCIPLINE.to_string(),
        writeback_policy: SEMANTIC_READ_MODEL_WRITEBACK_POLICY.to_string(),
        tables,
        required_steps: materialization_required_steps(expected_snapshot_revision.is_some()),
    })
}

fn materialization_required_steps(include_snapshot_gate: bool) -> Vec<String> {
    let mut steps = vec![
        "validate_semantic_repository".to_string(),
        "compute_read_model_snapshot".to_string(),
    ];
    if include_snapshot_gate {
        steps.push("check_expected_snapshot_revision".to_string());
    }
    steps.extend([
        "register_staging_tables".to_string(),
        "atomic_snapshot_swap".to_string(),
        "expose_read_only_queries".to_string(),
        "skip_semantic_artifact_writeback".to_string(),
    ]);
    steps
}
