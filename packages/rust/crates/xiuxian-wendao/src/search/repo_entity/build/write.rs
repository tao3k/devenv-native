use xiuxian_vector_store::VectorStoreError;

use crate::search::repo_entity::schema::{
    RepoEntityRow, path_column, repo_entity_batches, repo_entity_schema,
};
use crate::search::repo_publication_parquet::{
    ParquetPublicationStats, RepoPublicationRewriteRequest, inspect_repo_publication_parquet,
    rewrite_repo_publication_parquet,
};
use crate::search::{SearchCorpusKind, SearchPlaneService};

pub(crate) async fn write_replaced_table(
    service: &SearchPlaneService,
    table_name: &str,
    rows: &[RepoEntityRow],
) -> Result<ParquetPublicationStats, VectorStoreError> {
    let changed_batches = repo_entity_batches(rows)?;
    rewrite_repo_publication_parquet(
        service,
        RepoPublicationRewriteRequest {
            corpus: SearchCorpusKind::RepoEntity,
            base_table_name: None,
            target_table_name: table_name,
            path_column: path_column(),
            replaced_paths: &std::collections::BTreeSet::new(),
            changed_batches: changed_batches.as_slice(),
            empty_schema: Some(repo_entity_schema()),
        },
    )
    .await
}

pub(crate) async fn write_mutated_table(
    service: &SearchPlaneService,
    base_table_name: &str,
    target_table_name: &str,
    replaced_paths: &std::collections::BTreeSet<String>,
    changed_rows: &[RepoEntityRow],
) -> Result<ParquetPublicationStats, VectorStoreError> {
    let changed_batches = repo_entity_batches(changed_rows)?;
    rewrite_repo_publication_parquet(
        service,
        RepoPublicationRewriteRequest {
            corpus: SearchCorpusKind::RepoEntity,
            base_table_name: Some(base_table_name),
            target_table_name,
            path_column: path_column(),
            replaced_paths,
            changed_batches: changed_batches.as_slice(),
            empty_schema: Some(repo_entity_schema()),
        },
    )
    .await
}

pub(crate) async fn inspect_repo_entity_parquet(
    service: &SearchPlaneService,
    table_name: &str,
) -> Result<ParquetPublicationStats, VectorStoreError> {
    inspect_repo_publication_parquet(service, SearchCorpusKind::RepoEntity, table_name).await
}
