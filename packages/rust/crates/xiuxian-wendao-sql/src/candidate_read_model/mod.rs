//! DuckDB inspection over Episteme ontology candidate Parquet read models.

use std::path::PathBuf;

use duckdb::Connection;
use serde::{Deserialize, Serialize};

const INSPECTION_SCHEMA: &str = "xiuxian_wendao.sql.candidate_read_model_duckdb_inspection.v1";
const EXECUTION_ENGINE: &str = "duckdb";
const REGISTRATION_STRATEGY: &str = "duckdb_read_parquet_view";
const REVIEW_STATUS: &str = "review_required";
const PROMOTION_STATUS: &str = "blocked_pending_review";
const OBJECTS_PARQUET: &str = "ontology_candidate_objects.parquet";
const RELATIONS_PARQUET: &str = "ontology_candidate_relations.parquet";
const EVIDENCE_PARQUET: &str = "ontology_candidate_evidence.parquet";

/// Request for inspecting candidate Parquet read models through `DuckDB`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CandidateReadModelDuckDbInspectionRequest {
    /// Candidate objects Parquet path.
    pub objects: PathBuf,
    /// Candidate relations Parquet path.
    pub relations: PathBuf,
    /// Candidate evidence Parquet path.
    pub evidence: PathBuf,
}

impl CandidateReadModelDuckDbInspectionRequest {
    /// Create an inspection request from explicit Parquet paths.
    #[must_use]
    pub fn new(
        objects: impl Into<PathBuf>,
        relations: impl Into<PathBuf>,
        evidence: impl Into<PathBuf>,
    ) -> Self {
        Self {
            objects: objects.into(),
            relations: relations.into(),
            evidence: evidence.into(),
        }
    }

    /// Create an inspection request from the standard candidate run directory.
    #[must_use]
    pub fn from_candidate_run_dir(run_dir: impl Into<PathBuf>) -> Self {
        let run_dir = run_dir.into();
        Self {
            objects: run_dir.join(OBJECTS_PARQUET),
            relations: run_dir.join(RELATIONS_PARQUET),
            evidence: run_dir.join(EVIDENCE_PARQUET),
        }
    }
}

/// Count of rows grouped by one stable read-model kind.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateReadModelKindCount {
    /// Kind value from the read model.
    pub kind: String,
    /// Number of rows for this kind.
    pub row_count: usize,
}

/// Missing endpoint detected in the relation read model.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateReadModelMissingEndpoint {
    /// Relation candidate id whose endpoint is missing.
    pub relation_candidate_id: String,
    /// Endpoint role: `source` or `target`.
    pub endpoint_role: String,
    /// Referenced candidate id missing from object rows.
    pub endpoint_candidate_id: String,
}

/// `DuckDB` inspection report for candidate Parquet read models.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateReadModelDuckDbInspectionReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// SQL execution engine.
    pub execution_engine: &'static str,
    /// How Parquet tables were registered.
    pub registration_strategy: &'static str,
    /// Candidate object row count.
    pub candidate_object_count: usize,
    /// Candidate relation row count.
    pub candidate_relation_count: usize,
    /// Candidate evidence row count.
    pub candidate_evidence_count: usize,
    /// Object rows grouped by candidate kind.
    pub object_kind_counts: Vec<CandidateReadModelKindCount>,
    /// Relation rows grouped by relation kind.
    pub relation_kind_counts: Vec<CandidateReadModelKindCount>,
    /// Evidence rows grouped by evidence kind.
    pub evidence_kind_counts: Vec<CandidateReadModelKindCount>,
    /// Rows whose review status is not review-required.
    pub review_status_violation_count: usize,
    /// Rows whose promotion status is not blocked pending review.
    pub promotion_status_violation_count: usize,
    /// Rows that incorrectly claim ontology truth.
    pub ontology_truth_violation_count: usize,
    /// Object rows that incorrectly allow raw-to-RDF promotion.
    pub raw_to_rdf_promotion_violation_count: usize,
    /// Relation endpoints absent from object rows.
    pub missing_relation_endpoint_count: usize,
    /// Concrete relation endpoint violations.
    pub missing_relation_endpoints: Vec<CandidateReadModelMissingEndpoint>,
    /// Whether all inspection gates passed.
    pub inspection_passed: bool,
}

/// Inspect candidate Parquet read models through an in-memory `DuckDB` engine.
///
/// # Errors
///
/// Returns an error when `DuckDB` cannot open, register the Parquet files, or run
/// one of the fixed inspection queries.
pub fn inspect_candidate_read_model_with_duckdb(
    request: &CandidateReadModelDuckDbInspectionRequest,
) -> Result<CandidateReadModelDuckDbInspectionReport, String> {
    let connection = Connection::open_in_memory()
        .map_err(|error| format!("failed to open in-memory DuckDB connection: {error}"))?;
    register_candidate_views(&connection, request)?;

    let candidate_object_count = scalar_count(&connection, "select count(*) from objects")?;
    let candidate_relation_count = scalar_count(&connection, "select count(*) from relations")?;
    let candidate_evidence_count = scalar_count(&connection, "select count(*) from evidence")?;
    let object_kind_counts = kind_counts(
        &connection,
        "select candidate_kind, count(*) from objects group by candidate_kind order by candidate_kind",
    )?;
    let relation_kind_counts = kind_counts(
        &connection,
        "select relation_kind, count(*) from relations group by relation_kind order by relation_kind",
    )?;
    let evidence_kind_counts = kind_counts(
        &connection,
        "select evidence_kind, count(*) from evidence group by evidence_kind order by evidence_kind",
    )?;
    let review_status_violation_count = scalar_count(
        &connection,
        format!(
            "select count(*) from (
               select review_status from objects
               union all select review_status from relations
               union all select review_status from evidence
             ) where review_status <> {}",
            sql_string(REVIEW_STATUS)
        )
        .as_str(),
    )?;
    let promotion_status_violation_count = scalar_count(
        &connection,
        format!(
            "select count(*) from (
               select promotion_status from objects
               union all select promotion_status from relations
               union all select promotion_status from evidence
             ) where promotion_status <> {}",
            sql_string(PROMOTION_STATUS)
        )
        .as_str(),
    )?;
    let ontology_truth_violation_count = scalar_count(
        &connection,
        "select count(*) from (
           select ontology_truth from objects
           union all select ontology_truth from relations
           union all select ontology_truth from evidence
         ) where ontology_truth",
    )?;
    let raw_to_rdf_promotion_violation_count = scalar_count(
        &connection,
        "select count(*) from objects where raw_to_rdf_promotion_allowed",
    )?;
    let missing_relation_endpoints = missing_relation_endpoints(&connection)?;
    let inspection_passed = candidate_object_count > 0
        && review_status_violation_count == 0
        && promotion_status_violation_count == 0
        && ontology_truth_violation_count == 0
        && raw_to_rdf_promotion_violation_count == 0
        && missing_relation_endpoints.is_empty();

    Ok(CandidateReadModelDuckDbInspectionReport {
        schema_version: INSPECTION_SCHEMA,
        execution_engine: EXECUTION_ENGINE,
        registration_strategy: REGISTRATION_STRATEGY,
        candidate_object_count,
        candidate_relation_count,
        candidate_evidence_count,
        object_kind_counts,
        relation_kind_counts,
        evidence_kind_counts,
        review_status_violation_count,
        promotion_status_violation_count,
        ontology_truth_violation_count,
        raw_to_rdf_promotion_violation_count,
        missing_relation_endpoint_count: missing_relation_endpoints.len(),
        missing_relation_endpoints,
        inspection_passed,
    })
}

fn register_candidate_views(
    connection: &Connection,
    request: &CandidateReadModelDuckDbInspectionRequest,
) -> Result<(), String> {
    connection
        .execute_batch(
            format!(
                "create temp view objects as select * from read_parquet({});
                 create temp view relations as select * from read_parquet({});
                 create temp view evidence as select * from read_parquet({});",
                sql_string(request.objects.to_string_lossy().as_ref()),
                sql_string(request.relations.to_string_lossy().as_ref()),
                sql_string(request.evidence.to_string_lossy().as_ref())
            )
            .as_str(),
        )
        .map_err(|error| format!("failed to register candidate Parquet views: {error}"))
}

fn scalar_count(connection: &Connection, sql: &str) -> Result<usize, String> {
    let value = connection
        .query_row(sql, [], |row| row.get::<_, i64>(0))
        .map_err(|error| format!("failed to execute DuckDB count query `{sql}`: {error}"))?;
    usize::try_from(value).map_err(|_| format!("DuckDB count query `{sql}` returned {value}"))
}

fn kind_counts(
    connection: &Connection,
    sql: &str,
) -> Result<Vec<CandidateReadModelKindCount>, String> {
    let mut statement = connection
        .prepare(sql)
        .map_err(|error| format!("failed to plan DuckDB kind-count query: {error}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|error| format!("failed to execute DuckDB kind-count query: {error}"))?;
    rows.map(|row| {
        row.map_err(|error| format!("failed to read DuckDB kind-count row: {error}"))
            .and_then(|(kind, row_count)| {
                Ok(CandidateReadModelKindCount {
                    kind,
                    row_count: usize::try_from(row_count).map_err(|_| {
                        format!("DuckDB kind-count query returned negative count {row_count}")
                    })?,
                })
            })
    })
    .collect()
}

fn missing_relation_endpoints(
    connection: &Connection,
) -> Result<Vec<CandidateReadModelMissingEndpoint>, String> {
    let sql = "
        select r.candidate_id, 'source' as endpoint_role, r.source_candidate_id
        from relations r
        left join objects o on o.candidate_id = r.source_candidate_id
        where o.candidate_id is null
        union all
        select r.candidate_id, 'target' as endpoint_role, r.target_candidate_id
        from relations r
        left join objects o on o.candidate_id = r.target_candidate_id
        where o.candidate_id is null
        order by 1, 2, 3
    ";
    let mut statement = connection
        .prepare(sql)
        .map_err(|error| format!("failed to plan DuckDB endpoint query: {error}"))?;
    let rows = statement
        .query_map([], |row| {
            Ok(CandidateReadModelMissingEndpoint {
                relation_candidate_id: row.get(0)?,
                endpoint_role: row.get(1)?,
                endpoint_candidate_id: row.get(2)?,
            })
        })
        .map_err(|error| format!("failed to execute DuckDB endpoint query: {error}"))?;
    rows.map(|row| row.map_err(|error| format!("failed to read DuckDB endpoint row: {error}")))
        .collect()
}

fn sql_string(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(test)]
#[path = "../../tests/unit/candidate_read_model/mod.rs"]
mod tests;
