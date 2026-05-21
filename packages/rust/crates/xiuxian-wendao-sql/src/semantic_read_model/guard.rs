//! SQL guard checks for semantic read-model projection freshness.

use std::path::Path;

use serde_json::{Map, Value};
use xiuxian_wendao_parsers::semantic_ssot::{SemanticRepository, load_semantic_repository};

use crate::SqlQueryPayload;
use crate::local_relation::{DuckDbLocalRelationEngine, LocalRelationEngine};

use super::query::query_semantic_read_model_payload_with_engine;

/// Stable identifier for the first semantic SQL guard pilot.
pub const SEMANTIC_SQL_PROJECTION_FRESHNESS_GUARD_ID: &str = "semantic_sql.projection_freshness";
/// Semantic object that explains why projections remain read-model evidence.
pub const SEMANTIC_SQL_PROJECTION_FRESHNESS_OBJECT_ID: &str =
    "decision.semantic-ssot.projections-are-read-models";
/// SQL statement used by the projection freshness guard.
pub const SEMANTIC_SQL_PROJECTION_FRESHNESS_QUERY: &str = "\
select projection, source_revision, current_source_revision, projection_revision, staleness, source_path \
from semantic_projection_state \
where staleness <> 'fresh' \
order by projection";

/// Advisory status returned by one semantic SQL guard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticSqlGuardStatus {
    /// The SQL evidence found no rows that require review.
    Passed,
    /// The SQL evidence found rows that should be reviewed before relying on
    /// the projection.
    ReviewRequired,
}

impl SemanticSqlGuardStatus {
    /// Stable status label for rendered guard evidence.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::ReviewRequired => "review_required",
        }
    }
}

/// One stale projection row found by the semantic SQL guard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticProjectionFreshnessFinding {
    /// Projection name.
    pub projection: String,
    /// Declared source revision stored on the projection artifact.
    pub source_revision: String,
    /// Current source revision computed from source objects.
    pub current_source_revision: String,
    /// Projection revision identifier.
    pub projection_revision: String,
    /// Projection staleness token.
    pub staleness: String,
    /// Projection artifact path relative to the semantic root.
    pub source_path: String,
}

/// Advisory SQL guard evidence over the semantic read-model tables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticSqlGuardEvidence {
    /// Stable guard identifier.
    pub guard_id: String,
    /// Semantic object id that anchors the guard's architectural rationale.
    pub semantic_object_id: String,
    /// Advisory guard status.
    pub status: SemanticSqlGuardStatus,
    /// SQL statement used to derive the evidence.
    pub query_text: String,
    /// Count of rows that caused the guard to request review.
    pub failing_row_count: usize,
    /// Structured stale projection findings.
    pub findings: Vec<SemanticProjectionFreshnessFinding>,
    /// Human-readable summary message.
    pub message: String,
    /// Local relation engine label reported by the query payload.
    pub local_relation_engine: Option<String>,
}

/// Load a semantic repository and run the projection freshness SQL guard.
///
/// # Errors
///
/// Returns an error when semantic artifacts cannot be projected into the
/// read-model tables, the guard SQL query fails, or result rows do not match
/// the expected guard schema.
pub async fn run_semantic_sql_projection_freshness_guard(
    semantic_root: &Path,
) -> Result<SemanticSqlGuardEvidence, String> {
    let repository = load_semantic_repository(semantic_root);
    let query_engine = DuckDbLocalRelationEngine::new_in_memory()?;
    run_semantic_sql_projection_freshness_guard_with_engine(&repository, &query_engine).await
}

/// Run the projection freshness SQL guard against an already loaded repository.
///
/// # Errors
///
/// Returns an error when semantic artifacts cannot be projected into the
/// read-model tables, the guard SQL query fails, or result rows do not match
/// the expected guard schema.
pub async fn run_semantic_sql_projection_freshness_guard_with_engine(
    repository: &SemanticRepository,
    query_engine: &impl LocalRelationEngine,
) -> Result<SemanticSqlGuardEvidence, String> {
    let payload = query_semantic_read_model_payload_with_engine(
        repository,
        SEMANTIC_SQL_PROJECTION_FRESHNESS_QUERY,
        query_engine,
    )
    .await?;
    projection_freshness_evidence_from_payload(&payload)
}

fn projection_freshness_evidence_from_payload(
    payload: &SqlQueryPayload,
) -> Result<SemanticSqlGuardEvidence, String> {
    let findings = stale_projection_findings(payload)?;
    let failing_row_count = findings.len();
    let status = if failing_row_count == 0 {
        SemanticSqlGuardStatus::Passed
    } else {
        SemanticSqlGuardStatus::ReviewRequired
    };
    Ok(SemanticSqlGuardEvidence {
        guard_id: SEMANTIC_SQL_PROJECTION_FRESHNESS_GUARD_ID.to_string(),
        semantic_object_id: SEMANTIC_SQL_PROJECTION_FRESHNESS_OBJECT_ID.to_string(),
        status,
        query_text: SEMANTIC_SQL_PROJECTION_FRESHNESS_QUERY.to_string(),
        failing_row_count,
        findings,
        message: projection_freshness_message(status, failing_row_count),
        local_relation_engine: payload.metadata.local_relation_engine.clone(),
    })
}

fn stale_projection_findings(
    payload: &SqlQueryPayload,
) -> Result<Vec<SemanticProjectionFreshnessFinding>, String> {
    payload
        .batches
        .iter()
        .flat_map(|batch| batch.rows.iter())
        .map(stale_projection_finding_from_row)
        .collect::<Result<Vec<_>, _>>()
}

fn stale_projection_finding_from_row(
    row: &Map<String, Value>,
) -> Result<SemanticProjectionFreshnessFinding, String> {
    Ok(SemanticProjectionFreshnessFinding {
        projection: required_row_string(row, "projection")?,
        source_revision: required_row_string(row, "source_revision")?,
        current_source_revision: required_row_string(row, "current_source_revision")?,
        projection_revision: required_row_string(row, "projection_revision")?,
        staleness: required_row_string(row, "staleness")?,
        source_path: required_row_string(row, "source_path")?,
    })
}

fn required_row_string(row: &Map<String, Value>, key: &str) -> Result<String, String> {
    row.get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| format!("semantic SQL guard result row is missing string column `{key}`"))
}

fn projection_freshness_message(
    status: SemanticSqlGuardStatus,
    failing_row_count: usize,
) -> String {
    match status {
        SemanticSqlGuardStatus::Passed => {
            "semantic projection freshness guard passed: no stale projection rows".to_string()
        }
        SemanticSqlGuardStatus::ReviewRequired => {
            format!(
                "semantic projection freshness guard requires review: {failing_row_count} stale projection row(s)"
            )
        }
    }
}
