use std::path::Path;
use std::time::{Duration, Instant};

use arrow::record_batch::RecordBatch;
use serde::{Deserialize, Serialize};
use xiuxian_wendao_parsers::semantic_ssot::{SemanticRepository, load_semantic_repository};

use crate::local_relation::{DataFusionLocalRelationEngine, LocalRelationEngine};

use super::register::register_semantic_read_model_tables_with_stats;
use super::{
    SemanticReadModelSnapshot, semantic_read_model_snapshot, semantic_read_model_snapshot_check,
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

/// Smoke query used by the executable materialization preflight.
pub const SEMANTIC_READ_MODEL_MATERIALIZATION_PREFLIGHT_SMOKE_QUERY: &str = "select 'semantic_objects' as table_name, count(*) as row_count from semantic_objects \
     union all select 'semantic_relations' as table_name, count(*) as row_count from semantic_relations \
     union all select 'semantic_projection_state' as table_name, count(*) as row_count from semantic_projection_state \
     order by table_name";

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

/// Executable read-only materialization preflight report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticReadModelMaterializationPreflightReport {
    /// Plan that governs the attempted preflight.
    pub plan: SemanticReadModelMaterializationPlan,
    /// Execution details when the snapshot gate allows registration.
    pub execution: Option<SemanticReadModelMaterializationExecutionReport>,
}

/// Runtime execution details for a materialization preflight.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticReadModelMaterializationExecutionReport {
    /// Engine that executed the preflight.
    pub execution_engine: String,
    /// Number of registered read-model tables.
    pub registered_table_count: usize,
    /// Number of Arrow batches registered as read-model input.
    pub registered_input_batch_count: usize,
    /// Number of projected rows registered as read-model input.
    pub registered_input_row_count: usize,
    /// Approximate registered Arrow input bytes.
    pub registered_input_bytes: u64,
    /// Registration wall-clock time in milliseconds.
    pub registration_time_ms: u64,
    /// Read-only smoke query executed after registration.
    pub smoke_query: String,
    /// Number of batches returned by the smoke query.
    pub smoke_result_batch_count: usize,
    /// Number of rows returned by the smoke query.
    pub smoke_result_row_count: usize,
    /// Approximate smoke-query result bytes.
    pub smoke_result_bytes: u64,
    /// Smoke-query wall-clock time in milliseconds.
    pub smoke_query_time_ms: u64,
    /// Per-table runtime registration evidence.
    pub tables: Vec<SemanticReadModelMaterializationTablePreflight>,
}

/// Runtime preflight evidence for one semantic read-model table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticReadModelMaterializationTablePreflight {
    /// Table name exposed to read-only query consumers.
    pub name: String,
    /// Current projected row count.
    pub row_count: usize,
    /// Number of exposed columns.
    pub column_count: usize,
    /// Deterministic row revision for the table.
    pub row_revision: String,
    /// Runtime registration strategy.
    pub registration_strategy: String,
    /// Runtime materialization state.
    pub materialization_state: String,
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

/// Execute a read-only materialization preflight from a semantic artifact root.
///
/// # Errors
///
/// Returns an error when the semantic repository is invalid, the optional
/// expected snapshot revision is malformed, relation registration fails, or
/// the smoke query fails.
pub async fn semantic_read_model_materialization_preflight_from_root(
    root: impl AsRef<Path>,
    expected_snapshot_revision: Option<&str>,
) -> Result<SemanticReadModelMaterializationPreflightReport, String> {
    let repository = load_semantic_repository(root.as_ref());
    semantic_read_model_materialization_preflight(&repository, expected_snapshot_revision).await
}

/// Execute a read-only materialization preflight from a validated semantic repository.
///
/// # Errors
///
/// Returns an error when the semantic repository is invalid, the optional
/// expected snapshot revision is malformed, relation registration fails, or
/// the smoke query fails.
pub async fn semantic_read_model_materialization_preflight(
    repository: &SemanticRepository,
    expected_snapshot_revision: Option<&str>,
) -> Result<SemanticReadModelMaterializationPreflightReport, String> {
    let query_engine = DataFusionLocalRelationEngine::new_with_information_schema();
    semantic_read_model_materialization_preflight_with_engine(
        repository,
        expected_snapshot_revision,
        &query_engine,
    )
    .await
}

/// Execute a read-only materialization preflight using a supplied local relation engine.
///
/// # Errors
///
/// Returns an error when the semantic repository is invalid, the optional
/// expected snapshot revision is malformed, relation registration fails, or
/// the smoke query fails.
pub async fn semantic_read_model_materialization_preflight_with_engine(
    repository: &SemanticRepository,
    expected_snapshot_revision: Option<&str>,
    query_engine: &impl LocalRelationEngine,
) -> Result<SemanticReadModelMaterializationPreflightReport, String> {
    let snapshot = semantic_read_model_snapshot(repository)?;
    let plan = semantic_read_model_materialization_plan(snapshot, expected_snapshot_revision)?;
    if plan.status == SemanticReadModelMaterializationStatus::Blocked {
        return Ok(SemanticReadModelMaterializationPreflightReport {
            plan,
            execution: None,
        });
    }

    let registration_started_at = Instant::now();
    let registration = register_semantic_read_model_tables_with_stats(query_engine, repository)?;
    let registration_time_ms = duration_millis_u64(registration_started_at.elapsed());

    let smoke_query_started_at = Instant::now();
    let smoke_batches = query_engine
        .query_batches(SEMANTIC_READ_MODEL_MATERIALIZATION_PREFLIGHT_SMOKE_QUERY)
        .await?;
    let smoke_query_time_ms = duration_millis_u64(smoke_query_started_at.elapsed());
    let smoke_result_batch_count = smoke_batches.len();
    let smoke_result_row_count = smoke_batches.iter().map(RecordBatch::num_rows).sum();
    let smoke_result_bytes = batches_array_bytes(&smoke_batches);
    let tables = plan
        .tables
        .iter()
        .map(|table| SemanticReadModelMaterializationTablePreflight {
            name: table.name.clone(),
            row_count: table.row_count,
            column_count: table.column_count,
            row_revision: table.row_revision.clone(),
            registration_strategy: relation_registration_strategy(
                query_engine,
                table.name.as_str(),
            ),
            materialization_state: relation_materialization_state(
                query_engine,
                table.name.as_str(),
            ),
        })
        .collect::<Vec<_>>();

    Ok(SemanticReadModelMaterializationPreflightReport {
        plan,
        execution: Some(SemanticReadModelMaterializationExecutionReport {
            execution_engine: query_engine.kind().as_str().to_string(),
            registered_table_count: tables.len(),
            registered_input_batch_count: registration.input_batch_count,
            registered_input_row_count: registration.input_row_count,
            registered_input_bytes: registration.input_bytes,
            registration_time_ms,
            smoke_query: SEMANTIC_READ_MODEL_MATERIALIZATION_PREFLIGHT_SMOKE_QUERY.to_string(),
            smoke_result_batch_count,
            smoke_result_row_count,
            smoke_result_bytes,
            smoke_query_time_ms,
            tables,
        }),
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

fn relation_registration_strategy(
    query_engine: &impl LocalRelationEngine,
    table_name: &str,
) -> String {
    query_engine
        .relation_registration_strategy(table_name)
        .map_or_else(
            || format!("{}_request_scoped_arrow", query_engine.kind().as_str()),
            str::to_string,
        )
}

fn relation_materialization_state(
    query_engine: &impl LocalRelationEngine,
    table_name: &str,
) -> String {
    query_engine
        .relation_materialization_state(table_name)
        .map_or_else(|| "unknown".to_string(), |state| state.as_str().to_string())
}

fn duration_millis_u64(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn batches_array_bytes(batches: &[RecordBatch]) -> u64 {
    batches.iter().fold(0_u64, |total, batch| {
        total.saturating_add(u64::try_from(batch.get_array_memory_size()).unwrap_or(u64::MAX))
    })
}
