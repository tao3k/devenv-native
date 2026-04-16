use std::collections::BTreeSet;

use xiuxian_vector_store::VectorStoreError;

use crate::repo_index::RepoCodeDocument;
use crate::search::repo_content_chunk::schema::{
    path_column, repo_content_chunk_batches, repo_content_chunk_schema, rows_from_documents,
};
use crate::search::repo_publication_parquet::{
    ParquetPublicationStats, RepoPublicationRewriteRequest, inspect_repo_publication_parquet,
    rewrite_repo_publication_parquet,
};
use crate::search::{SearchCorpusKind, SearchPlaneService};

pub(crate) async fn write_replaced_table(
    service: &SearchPlaneService,
    table_name: &str,
    documents: &[RepoCodeDocument],
) -> Result<ParquetPublicationStats, VectorStoreError> {
    let rows = rows_from_documents(documents);
    let changed_batches = repo_content_chunk_batches(&rows)?;
    rewrite_repo_publication_parquet(
        service,
        RepoPublicationRewriteRequest {
            corpus: SearchCorpusKind::RepoContentChunk,
            base_table_name: None,
            target_table_name: table_name,
            path_column: path_column(),
            replaced_paths: &BTreeSet::new(),
            changed_batches: changed_batches.as_slice(),
            empty_schema: Some(repo_content_chunk_schema()),
        },
    )
    .await
}

pub(crate) async fn write_mutated_table(
    service: &SearchPlaneService,
    base_table_name: &str,
    target_table_name: &str,
    replaced_paths: &BTreeSet<String>,
    changed_documents: &[RepoCodeDocument],
) -> Result<ParquetPublicationStats, VectorStoreError> {
    let changed_rows = rows_from_documents(changed_documents);
    let changed_batches = repo_content_chunk_batches(&changed_rows)?;
    rewrite_repo_publication_parquet(
        service,
        RepoPublicationRewriteRequest {
            corpus: SearchCorpusKind::RepoContentChunk,
            base_table_name: Some(base_table_name),
            target_table_name,
            path_column: path_column(),
            replaced_paths,
            changed_batches: changed_batches.as_slice(),
            empty_schema: Some(repo_content_chunk_schema()),
        },
    )
    .await
}

pub(crate) async fn inspect_repo_content_chunk_parquet(
    service: &SearchPlaneService,
    table_name: &str,
) -> Result<ParquetPublicationStats, VectorStoreError> {
    inspect_repo_publication_parquet(service, SearchCorpusKind::RepoContentChunk, table_name).await
}
