use std::collections::BTreeMap;
use std::collections::BTreeSet;

use xiuxian_vector_store::VectorStoreError;

use crate::repo_index::RepoCodeDocument;
use crate::search::cache::SearchPlaneFileFingerprintScope;
use crate::search::repo_content_chunk::build::plan::{
    merge_repo_content_chunk_file_fingerprints, plan_repo_content_chunk_build,
    plan_repo_content_chunk_incremental_build,
};
use crate::search::repo_content_chunk::build::types::{
    RepoContentChunkBuildAction, RepoContentChunkBuildPlan,
};
use crate::search::repo_content_chunk::build::write::{
    inspect_repo_content_chunk_parquet, write_mutated_table, write_replaced_table,
};
use crate::search::repo_content_chunk::schema::projected_columns;
use crate::search::{
    SearchCorpusKind, SearchFileFingerprint, SearchPlaneService, SearchPublicationStorageFormat,
    SearchRepoPublicationInput,
};

pub(crate) async fn publish_repo_content_chunks(
    service: &SearchPlaneService,
    repo_id: &str,
    documents: &[RepoCodeDocument],
    source_revision: Option<&str>,
) -> Result<(), VectorStoreError> {
    let previous_fingerprints = service
        .file_fingerprints(SearchPlaneFileFingerprintScope::repo_corpus(
            SearchCorpusKind::RepoContentChunk,
            repo_id,
        ))
        .await;
    let current_record = service
        .repo_corpus_record_for_reads(SearchCorpusKind::RepoContentChunk, repo_id)
        .await;
    let plan = plan_repo_content_chunk_build(
        repo_id,
        documents,
        source_revision,
        current_record
            .as_ref()
            .and_then(|record| record.publication.as_ref()),
        &previous_fingerprints,
    );

    apply_repo_content_chunk_build_plan(service, repo_id, source_revision, &plan).await
}

pub(crate) async fn publish_repo_content_chunks_incremental(
    service: &SearchPlaneService,
    repo_id: &str,
    changed_documents: &[RepoCodeDocument],
    deleted_paths: &BTreeSet<String>,
    source_revision: Option<&str>,
) -> Result<(), VectorStoreError> {
    let previous_fingerprints = service
        .file_fingerprints(SearchPlaneFileFingerprintScope::repo_corpus(
            SearchCorpusKind::RepoContentChunk,
            repo_id,
        ))
        .await;
    let current_record = service
        .repo_corpus_record_for_reads(SearchCorpusKind::RepoContentChunk, repo_id)
        .await;
    let Some(previous_publication) = current_record
        .as_ref()
        .and_then(|record| record.publication.as_ref())
    else {
        return Err(VectorStoreError::General(format!(
            "repo `{repo_id}` incremental repo-content publish requires an existing publication"
        )));
    };
    let file_fingerprints = merge_repo_content_chunk_file_fingerprints(
        &previous_fingerprints,
        changed_documents,
        deleted_paths,
    );
    let plan = plan_repo_content_chunk_incremental_build(
        repo_id,
        changed_documents,
        &file_fingerprints,
        source_revision,
        Some(previous_publication),
        &previous_fingerprints,
    );

    if matches!(plan.action, RepoContentChunkBuildAction::ReplaceAll { .. }) {
        return Err(VectorStoreError::General(format!(
            "repo `{repo_id}` incremental repo-content publish unexpectedly requested replace-all"
        )));
    }
    apply_repo_content_chunk_build_plan(service, repo_id, source_revision, &plan).await
}

async fn apply_repo_content_chunk_build_plan(
    service: &SearchPlaneService,
    repo_id: &str,
    source_revision: Option<&str>,
    plan: &RepoContentChunkBuildPlan,
) -> Result<(), VectorStoreError> {
    match &plan.action {
        RepoContentChunkBuildAction::Noop => {
            set_repo_content_chunk_file_fingerprints(service, repo_id, &plan.file_fingerprints)
                .await;
            Ok(())
        }
        RepoContentChunkBuildAction::RefreshPublication { table_name } => {
            refresh_repo_content_chunk_publication(
                service,
                repo_id,
                table_name.as_str(),
                source_revision,
                &plan.file_fingerprints,
            )
            .await
        }
        RepoContentChunkBuildAction::ReplaceAll {
            table_name,
            payload: documents,
        } => {
            let parquet_stats =
                write_replaced_table(service, table_name.as_str(), documents).await?;
            finalize_repo_content_publication(
                service,
                repo_id,
                table_name.as_str(),
                source_revision,
                parquet_stats,
                &plan.file_fingerprints,
            )
            .await
        }
        RepoContentChunkBuildAction::CloneAndMutate {
            base_table_name,
            target_table_name,
            replaced_paths,
            changed_payload: changed_documents,
        } => {
            let parquet_stats = write_mutated_table(
                service,
                base_table_name.as_str(),
                target_table_name.as_str(),
                replaced_paths,
                changed_documents,
            )
            .await?;
            finalize_repo_content_publication(
                service,
                repo_id,
                target_table_name.as_str(),
                source_revision,
                parquet_stats,
                &plan.file_fingerprints,
            )
            .await
        }
    }
}

async fn refresh_repo_content_chunk_publication(
    service: &SearchPlaneService,
    repo_id: &str,
    table_name: &str,
    source_revision: Option<&str>,
    file_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) -> Result<(), VectorStoreError> {
    let parquet_stats = inspect_repo_content_chunk_parquet(service, table_name).await?;
    service
        .record_repo_publication_input_with_storage_format(
            SearchCorpusKind::RepoContentChunk,
            repo_id,
            SearchRepoPublicationInput {
                table_name: table_name.to_string(),
                schema_version: SearchCorpusKind::RepoContentChunk.schema_version(),
                source_revision: source_revision.map(str::to_string),
                table_version_id: parquet_stats.table_version_id,
                row_count: parquet_stats.row_count,
                fragment_count: parquet_stats.fragment_count,
                published_at: parquet_stats.published_at,
            },
            SearchPublicationStorageFormat::Parquet,
        )
        .await;
    set_repo_content_chunk_file_fingerprints(service, repo_id, file_fingerprints).await;
    Ok(())
}

async fn set_repo_content_chunk_file_fingerprints(
    service: &SearchPlaneService,
    repo_id: &str,
    file_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) {
    service
        .set_file_fingerprints(
            SearchPlaneFileFingerprintScope::repo_corpus(
                SearchCorpusKind::RepoContentChunk,
                repo_id,
            ),
            file_fingerprints,
        )
        .await;
}

async fn finalize_repo_content_publication(
    service: &SearchPlaneService,
    repo_id: &str,
    table_name: &str,
    source_revision: Option<&str>,
    parquet_stats: crate::search::repo_publication_parquet::ParquetPublicationStats,
    file_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) -> Result<(), VectorStoreError> {
    let prewarm_columns = projected_columns();
    service
        .prewarm_repo_table(
            SearchCorpusKind::RepoContentChunk,
            repo_id,
            table_name,
            &prewarm_columns,
        )
        .await?;
    service
        .record_repo_publication_input_with_storage_format(
            SearchCorpusKind::RepoContentChunk,
            repo_id,
            SearchRepoPublicationInput {
                table_name: table_name.to_string(),
                schema_version: SearchCorpusKind::RepoContentChunk.schema_version(),
                source_revision: source_revision.map(str::to_string),
                table_version_id: parquet_stats.table_version_id,
                row_count: parquet_stats.row_count,
                fragment_count: parquet_stats.fragment_count,
                published_at: parquet_stats.published_at,
            },
            SearchPublicationStorageFormat::Parquet,
        )
        .await;
    service
        .set_file_fingerprints(
            SearchPlaneFileFingerprintScope::repo_corpus(
                SearchCorpusKind::RepoContentChunk,
                repo_id,
            ),
            file_fingerprints,
        )
        .await;
    Ok(())
}
