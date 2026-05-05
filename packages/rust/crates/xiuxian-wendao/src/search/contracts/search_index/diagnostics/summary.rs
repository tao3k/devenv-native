//! Aggregate diagnostic summaries built from local relation queries.

use super::decode::{
    decode_query_telemetry_summary, decode_repo_read_pressure_summary,
    decode_status_reason_summary, decode_status_rollup,
};
use super::relations::{
    query_telemetry_relation, repo_read_pressure_relation, status_reason_relation,
    status_snapshot_relation,
};
use super::sql::{
    QUERY_TELEMETRY_SCOPE_SQL, QUERY_TELEMETRY_SUMMARY_SQL, REPO_READ_PRESSURE_SUMMARY_SQL,
    STATUS_DIAGNOSTICS_SQL, STATUS_REASON_SUMMARY_SQL,
};
#[cfg(all(test, feature = "duckdb"))]
use crate::duckdb::LocalRelationEngineKind;
use crate::duckdb::{
    DataFusionLocalRelationEngine, LocalRelationEngine, LocalRelationRegistrationHint,
};
#[cfg(feature = "duckdb")]
use crate::duckdb::{DuckDbLocalRelationEngine, resolve_search_duckdb_runtime};
use crate::search::SearchPlaneStatusSnapshot;
use crate::search::contracts::search_index::definitions as search_index;

const STATUS_DIAGNOSTICS_TABLE: &str = "status_rollup_rows";
const STATUS_REASON_DIAGNOSTICS_TABLE: &str = "status_reason_rows";
const QUERY_TELEMETRY_DIAGNOSTICS_TABLE: &str = "query_telemetry_rows";
const REPO_READ_PRESSURE_DIAGNOSTICS_TABLE: &str = "repo_read_pressure_rows";

pub(crate) struct SearchIndexDiagnosticsSummary {
    pub(crate) rollup: SearchIndexDiagnosticsRollup,
    pub(crate) status_reason: Option<search_index::SearchIndexAggregateStatusReason>,
    pub(crate) query_telemetry_summary: Option<search_index::SearchIndexAggregateQueryTelemetry>,
    pub(crate) repo_read_pressure: Option<search_index::SearchIndexRepoReadPressure>,
}

pub(crate) struct SearchIndexDiagnosticsRollup {
    pub(crate) total: usize,
    pub(crate) idle: usize,
    pub(crate) indexing: usize,
    pub(crate) ready: usize,
    pub(crate) degraded: usize,
    pub(crate) failed: usize,
    pub(crate) compaction_pending: usize,
    pub(crate) maintenance_summary: Option<search_index::SearchIndexAggregateMaintenanceSummary>,
}

#[cfg(all(test, feature = "duckdb"))]
pub(crate) fn configured_status_diagnostics_engine_kind() -> Result<LocalRelationEngineKind, String>
{
    Ok(configured_status_diagnostics_engine()?.kind())
}

pub(crate) async fn summarize_status_diagnostics(
    snapshot: &SearchPlaneStatusSnapshot,
) -> Result<SearchIndexDiagnosticsSummary, String> {
    #[cfg(feature = "duckdb")]
    let engine = configured_status_diagnostics_engine()?;
    #[cfg(not(feature = "duckdb"))]
    let engine = configured_status_diagnostics_engine();
    let (schema, batches) = status_snapshot_relation(snapshot)?;
    engine.register_record_batches(STATUS_DIAGNOSTICS_TABLE, schema, batches)?;
    let rollup_batches = engine
        .query_batches(STATUS_DIAGNOSTICS_SQL)
        .await
        .map_err(|error| format!("status diagnostics rollup query failed: {error}"))?;
    let rollup = decode_status_rollup(rollup_batches.as_slice())?;
    let status_reason = summarize_status_reason_diagnostics(snapshot, engine.as_ref()).await?;
    let query_telemetry_summary =
        summarize_query_telemetry_diagnostics(snapshot, engine.as_ref()).await?;
    let repo_read_pressure =
        summarize_repo_read_pressure_diagnostics(snapshot, engine.as_ref()).await?;
    Ok(SearchIndexDiagnosticsSummary {
        rollup,
        status_reason,
        query_telemetry_summary,
        repo_read_pressure,
    })
}

#[cfg(feature = "duckdb")]
fn configured_status_diagnostics_engine() -> Result<Box<dyn LocalRelationEngine>, String> {
    #[cfg(feature = "duckdb")]
    {
        let runtime = resolve_search_duckdb_runtime();
        if runtime.enabled {
            return DuckDbLocalRelationEngine::from_runtime(runtime)
                .map(|engine| Box::new(engine) as Box<dyn LocalRelationEngine>);
        }
    }

    Ok(Box::new(
        DataFusionLocalRelationEngine::new_with_information_schema(),
    ))
}

#[cfg(not(feature = "duckdb"))]
fn configured_status_diagnostics_engine() -> Box<dyn LocalRelationEngine> {
    Box::new(DataFusionLocalRelationEngine::new_with_information_schema())
}

async fn summarize_query_telemetry_diagnostics(
    snapshot: &SearchPlaneStatusSnapshot,
    engine: &dyn LocalRelationEngine,
) -> Result<Option<search_index::SearchIndexAggregateQueryTelemetry>, String> {
    let Some((schema, batches)) = query_telemetry_relation(snapshot)? else {
        return Ok(None);
    };
    engine.register_record_batches_with_hint(
        QUERY_TELEMETRY_DIAGNOSTICS_TABLE,
        schema,
        batches,
        LocalRelationRegistrationHint::RepeatedUse,
    )?;
    let summary_batches = engine
        .query_batches(QUERY_TELEMETRY_SUMMARY_SQL)
        .await
        .map_err(|error| format!("status diagnostics query telemetry summary failed: {error}"))?;
    let scopes_batches = engine
        .query_batches(QUERY_TELEMETRY_SCOPE_SQL)
        .await
        .map_err(|error| {
            format!("status diagnostics query telemetry scope query failed: {error}")
        })?;
    Ok(Some(decode_query_telemetry_summary(
        summary_batches.as_slice(),
        scopes_batches.as_slice(),
    )?))
}

async fn summarize_status_reason_diagnostics(
    snapshot: &SearchPlaneStatusSnapshot,
    engine: &dyn LocalRelationEngine,
) -> Result<Option<search_index::SearchIndexAggregateStatusReason>, String> {
    let Some((schema, batches)) = status_reason_relation(snapshot)? else {
        return Ok(None);
    };
    engine.register_record_batches(STATUS_REASON_DIAGNOSTICS_TABLE, schema, batches)?;
    let summary_batches = engine
        .query_batches(STATUS_REASON_SUMMARY_SQL)
        .await
        .map_err(|error| format!("status diagnostics status reason query failed: {error}"))?;
    decode_status_reason_summary(summary_batches.as_slice())
}

async fn summarize_repo_read_pressure_diagnostics(
    snapshot: &SearchPlaneStatusSnapshot,
    engine: &dyn LocalRelationEngine,
) -> Result<Option<search_index::SearchIndexRepoReadPressure>, String> {
    let Some((schema, batches)) = repo_read_pressure_relation(snapshot)? else {
        return Ok(None);
    };
    engine.register_record_batches(REPO_READ_PRESSURE_DIAGNOSTICS_TABLE, schema, batches)?;
    let summary_batches = engine
        .query_batches(REPO_READ_PRESSURE_SUMMARY_SQL)
        .await
        .map_err(|error| format!("status diagnostics repo read pressure query failed: {error}"))?;
    decode_repo_read_pressure_summary(summary_batches.as_slice())
}
