//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use std::collections::BTreeMap;

use crate::duckdb::ParquetQueryEngine;
use crate::search::repo_entity::query::hydrate::{
    load_hydrated_rows_by_id, typed_repo_entity_columns,
};
use crate::search::repo_entity::query::lookup::{
    HydratedRepoEntityRow, RepoEntityCandidate, RepoEntitySearchError,
};

pub(super) async fn load_hydrated_rows(
    query_engine: &ParquetQueryEngine,
    engine_table_name: &str,
    candidates: &[RepoEntityCandidate],
) -> Result<BTreeMap<String, HydratedRepoEntityRow>, RepoEntitySearchError> {
    let ids = candidates
        .iter()
        .map(|candidate| candidate.id.clone())
        .collect::<Vec<_>>();
    load_hydrated_rows_by_id(
        query_engine,
        engine_table_name,
        ids.as_slice(),
        typed_repo_entity_columns().as_slice(),
    )
    .await
}
