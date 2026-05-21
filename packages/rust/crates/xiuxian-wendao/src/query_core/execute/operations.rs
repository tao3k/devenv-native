//! `query_core::execute::operations` owns Wendao query core execute operations behavior.

use crate::query_core::context::WendaoExecutionContext;
use crate::query_core::operators::{
    ColumnMaskOp, ColumnMaskPredicate, GraphNeighborsOp, PayloadFetchOp, VectorSearchOp,
};
use crate::query_core::telemetry::WendaoExplainEvent;
use crate::query_core::types::{
    WendaoBackendKind, WendaoOperatorKind, WendaoQueryCoreError, WendaoRelation,
};

/// Execute a retrieval-first search via the configured retrieval backend.
///
/// # Errors
///
/// Returns an error when the retrieval backend is missing or when the backend
/// cannot materialize the requested relation.
pub async fn execute_vector_search(
    ctx: &WendaoExecutionContext,
    op: &VectorSearchOp,
) -> Result<WendaoRelation, WendaoQueryCoreError> {
    let backend = ctx
        .retrieval_backend
        .as_ref()
        .ok_or(WendaoQueryCoreError::MissingBackend("retrieval"))?;
    let relation = backend.vector_search(op).await?;
    ctx.explain_sink.record(WendaoExplainEvent {
        operator_kind: WendaoOperatorKind::VectorSearch,
        backend_kind: WendaoBackendKind::SearchPlaneBackend,
        legacy_adapter: true,
        input_row_count: None,
        output_row_count: Some(relation.row_count()),
        payload_fetch: false,
        narrow_phase_surviving_count: None,
        payload_phase_fetched_count: None,
        note: Some("search-plane backend".to_string()),
    });
    Ok(relation)
}

/// Execute a graph-neighbor lookup via the configured graph backend.
///
/// # Errors
///
/// Returns an error when the graph backend is missing or when the backend
/// cannot materialize the requested relation.
pub async fn execute_graph_neighbors(
    ctx: &WendaoExecutionContext,
    op: &GraphNeighborsOp,
) -> Result<WendaoRelation, WendaoQueryCoreError> {
    let backend = ctx
        .graph_backend
        .as_ref()
        .ok_or(WendaoQueryCoreError::MissingBackend("graph"))?;
    let relation = backend.graph_neighbors(op).await?;
    ctx.explain_sink.record(WendaoExplainEvent {
        operator_kind: WendaoOperatorKind::GraphNeighbors,
        backend_kind: WendaoBackendKind::LinkGraphBackend,
        legacy_adapter: true,
        input_row_count: None,
        output_row_count: Some(relation.row_count()),
        payload_fetch: false,
        narrow_phase_surviving_count: None,
        payload_phase_fetched_count: None,
        note: Some("link-graph backend".to_string()),
    });
    Ok(relation)
}

/// Execute a narrow-column filter before payload hydration.
///
/// # Errors
///
/// Returns an error when the input relation cannot be decoded into retrieval
/// rows or when the filtered rows cannot be encoded back into a relation.
pub fn execute_column_mask(
    ctx: &WendaoExecutionContext,
    op: &ColumnMaskOp,
) -> Result<WendaoRelation, WendaoQueryCoreError> {
    let mut rows = Vec::new();
    for batch in op.relation.batches() {
        rows.extend(xiuxian_db_store::retrieval_rows_from_record_batch(batch)?);
    }
    let input_row_count = rows.len();

    for predicate in &op.predicates {
        rows.retain(|row| match predicate {
            ColumnMaskPredicate::IdIn(ids) => ids.contains(&row.id),
            ColumnMaskPredicate::RepoEquals(repo) => row.repo.as_deref() == Some(repo.as_str()),
            ColumnMaskPredicate::PathContains(fragment) => row.path.contains(fragment),
            ColumnMaskPredicate::ScoreAtLeast(min_score) => {
                row.score.unwrap_or_default() >= *min_score
            }
        });
    }
    if let Some(limit) = op.limit {
        rows.truncate(limit);
    }

    let batch = xiuxian_db_store::retrieval_rows_to_record_batch(&rows)?;
    let relation = WendaoRelation::new(batch.schema(), vec![batch]);
    ctx.explain_sink.record(WendaoExplainEvent {
        operator_kind: WendaoOperatorKind::ColumnMask,
        backend_kind: WendaoBackendKind::QueryCoreMask,
        legacy_adapter: false,
        input_row_count: Some(input_row_count),
        output_row_count: Some(relation.row_count()),
        payload_fetch: false,
        narrow_phase_surviving_count: Some(relation.row_count()),
        payload_phase_fetched_count: None,
        note: Some("narrow-column mask".to_string()),
    });
    Ok(relation)
}

/// Execute payload hydration and projection via the retrieval backend.
///
/// # Errors
///
/// Returns an error when the retrieval backend is missing or when payload
/// projection fails for the requested relation.
pub async fn execute_payload_fetch(
    ctx: &WendaoExecutionContext,
    op: &PayloadFetchOp,
) -> Result<WendaoRelation, WendaoQueryCoreError> {
    let backend = ctx
        .retrieval_backend
        .as_ref()
        .ok_or(WendaoQueryCoreError::MissingBackend("retrieval"))?;
    let input_row_count = op.relation.row_count();
    let relation = backend.payload_fetch(&op.relation, op).await?;
    ctx.explain_sink.record(WendaoExplainEvent {
        operator_kind: WendaoOperatorKind::PayloadFetch,
        backend_kind: WendaoBackendKind::VectorRetrievalAdapter,
        legacy_adapter: true,
        input_row_count: Some(input_row_count),
        output_row_count: Some(relation.row_count()),
        payload_fetch: true,
        narrow_phase_surviving_count: None,
        payload_phase_fetched_count: Some(relation.row_count()),
        note: Some("retrieval payload projection".to_string()),
    });
    Ok(relation)
}
