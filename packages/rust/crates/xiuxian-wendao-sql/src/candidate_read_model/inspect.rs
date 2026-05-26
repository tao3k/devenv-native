//! `DuckDB` inspection implementation for candidate read-model Parquet files.

use duckdb::Connection;

use super::constants::{
    EXECUTION_ENGINE, INSPECTION_SCHEMA, PROMOTION_STATUS, REGISTRATION_STRATEGY, REVIEW_STATUS,
};
use super::types::{
    CandidateReadModelDuckDbInspectionReport, CandidateReadModelDuckDbInspectionRequest,
    CandidateReadModelKindCount, CandidateReadModelMissingEndpoint,
};

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

    let counts = candidate_read_model_counts(&connection)?;
    let review_status_violation_count =
        status_violation_count(&connection, "review_status", REVIEW_STATUS, "review status")?;
    let promotion_status_violation_count = status_violation_count(
        &connection,
        "promotion_status",
        PROMOTION_STATUS,
        "promotion status",
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
    let inspection_passed = counts.candidate_object_count > 0
        && review_status_violation_count == 0
        && promotion_status_violation_count == 0
        && ontology_truth_violation_count == 0
        && raw_to_rdf_promotion_violation_count == 0
        && missing_relation_endpoints.is_empty();

    Ok(CandidateReadModelDuckDbInspectionReport {
        schema_version: INSPECTION_SCHEMA,
        execution_engine: EXECUTION_ENGINE,
        registration_strategy: REGISTRATION_STRATEGY,
        candidate_object_count: counts.candidate_object_count,
        candidate_relation_count: counts.candidate_relation_count,
        candidate_evidence_count: counts.candidate_evidence_count,
        object_kind_counts: counts.object_kind_counts,
        relation_kind_counts: counts.relation_kind_counts,
        evidence_kind_counts: counts.evidence_kind_counts,
        review_status_violation_count,
        promotion_status_violation_count,
        ontology_truth_violation_count,
        raw_to_rdf_promotion_violation_count,
        missing_relation_endpoint_count: missing_relation_endpoints.len(),
        missing_relation_endpoints,
        inspection_passed,
    })
}

struct CandidateReadModelCounts {
    candidate_object_count: usize,
    candidate_relation_count: usize,
    candidate_evidence_count: usize,
    object_kind_counts: Vec<CandidateReadModelKindCount>,
    relation_kind_counts: Vec<CandidateReadModelKindCount>,
    evidence_kind_counts: Vec<CandidateReadModelKindCount>,
}

fn candidate_read_model_counts(
    connection: &Connection,
) -> Result<CandidateReadModelCounts, String> {
    Ok(CandidateReadModelCounts {
        candidate_object_count: scalar_count(connection, "select count(*) from objects")?,
        candidate_relation_count: scalar_count(connection, "select count(*) from relations")?,
        candidate_evidence_count: scalar_count(connection, "select count(*) from evidence")?,
        object_kind_counts: kind_counts(
            connection,
            "select candidate_kind, count(*) from objects group by candidate_kind order by candidate_kind",
        )?,
        relation_kind_counts: kind_counts(
            connection,
            "select relation_kind, count(*) from relations group by relation_kind order by relation_kind",
        )?,
        evidence_kind_counts: kind_counts(
            connection,
            "select evidence_kind, count(*) from evidence group by evidence_kind order by evidence_kind",
        )?,
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

fn status_violation_count(
    connection: &Connection,
    status_column: &str,
    expected_status: &str,
    label: &str,
) -> Result<usize, String> {
    scalar_count(
        connection,
        format!(
            "select count(*) from (
               select {status_column} from objects
               union all select {status_column} from relations
               union all select {status_column} from evidence
             ) where {status_column} <> {}",
            sql_string(expected_status)
        )
        .as_str(),
    )
    .map_err(|error| format!("failed to inspect candidate {label}: {error}"))
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
                    kind: kind.into(),
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
