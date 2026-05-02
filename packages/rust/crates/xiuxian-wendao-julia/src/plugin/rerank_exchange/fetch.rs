//! Fetch helpers for Julia rerank exchange routes.

use std::collections::BTreeMap;

use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::{RegisteredRepository, RepoIntelligenceError};
use xiuxian_wendao_runtime::transport::{PluginArrowScoreRow, decode_plugin_arrow_score_rows};

use crate::plugin::transport::process_julia_flight_batches_for_repository;

/// Execute the repository-configured Julia Flight transport and materialize the
/// validated response into typed score rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the remote roundtrip fails or the
/// decoded response cannot be materialized into the `WendaoArrow` `v1` score row
/// contract.
pub async fn fetch_plugin_arrow_score_rows_for_repository(
    repository: &RegisteredRepository,
    batches: &[RecordBatch],
) -> Result<BTreeMap<String, PluginArrowScoreRow>, RepoIntelligenceError> {
    let response_batches = process_julia_flight_batches_for_repository(repository, batches).await?;
    decode_plugin_arrow_score_rows(response_batches.as_slice())
}

/// Execute the Julia Flight score-row fetch path under the Julia-owned surface.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the remote roundtrip fails or the
/// decoded response cannot be materialized into the `WendaoArrow` `v1` score row
/// contract.
pub async fn fetch_julia_flight_score_rows_for_repository(
    repository: &RegisteredRepository,
    batches: &[RecordBatch],
) -> Result<BTreeMap<String, PluginArrowScoreRow>, RepoIntelligenceError> {
    fetch_plugin_arrow_score_rows_for_repository(repository, batches).await
}
