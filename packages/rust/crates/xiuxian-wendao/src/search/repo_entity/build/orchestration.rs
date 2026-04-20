use crate::analyzers::RepositoryAnalysisOutput;
use crate::repo_index::RepoCodeDocument;
use crate::search::cache::SearchPlaneFileFingerprintScope;
use crate::search::repo_entity::build::RepoEntityBuildPlan;
use crate::search::repo_entity::build::plan_repo_entity_build;
use crate::search::repo_entity::schema::{hit_json_column, projected_columns, rows_from_analysis};
use crate::search::{
    SearchCorpusKind, SearchPlaneService, SearchPublicationStorageFormat,
    SearchRepoPublicationInput,
};

use crate::search::repo_entity::build::RepoEntityBuildAction;
use crate::search::repo_entity::build::write::{
    inspect_repo_entity_parquet, write_mutated_table, write_replaced_table,
};

use std::collections::BTreeMap;
use xiuxian_db_store::VectorStoreError;

pub(crate) async fn publish_repo_entities(
    service: &SearchPlaneService,
    repo_id: &str,
    analysis: &RepositoryAnalysisOutput,
    documents: &[RepoCodeDocument],
    source_revision: Option<&str>,
) -> Result<(), VectorStoreError> {
    let previous_fingerprints = service
        .file_fingerprints(SearchPlaneFileFingerprintScope::repo_corpus(
            SearchCorpusKind::RepoEntity,
            repo_id,
        ))
        .await;
    let current_record = service
        .repo_corpus_record_for_reads(SearchCorpusKind::RepoEntity, repo_id)
        .await;
    let rows = rows_from_analysis(repo_id, analysis)?;
    let plan = plan_repo_entity_build(
        repo_id,
        &rows,
        documents,
        source_revision,
        current_record
            .as_ref()
            .and_then(|record| record.publication.as_ref()),
        &previous_fingerprints,
    );

    apply_repo_entity_build_plan(service, repo_id, source_revision, &plan).await
}

async fn apply_repo_entity_build_plan(
    service: &SearchPlaneService,
    repo_id: &str,
    source_revision: Option<&str>,
    plan: &RepoEntityBuildPlan,
) -> Result<(), VectorStoreError> {
    match &plan.action {
        RepoEntityBuildAction::Noop => {
            set_repo_entity_file_fingerprints(service, repo_id, &plan.file_fingerprints).await;
            Ok(())
        }
        RepoEntityBuildAction::RefreshPublication { table_name } => {
            refresh_repo_entity_publication(
                service,
                repo_id,
                table_name.as_str(),
                source_revision,
                &plan.file_fingerprints,
            )
            .await
        }
        RepoEntityBuildAction::ReplaceAll {
            table_name,
            payload: rows,
        } => {
            let parquet_stats = write_replaced_table(service, table_name.as_str(), rows).await?;
            finalize_repo_entity_publication(
                service,
                repo_id,
                table_name.as_str(),
                source_revision,
                parquet_stats,
                &plan.file_fingerprints,
            )
            .await
        }
        RepoEntityBuildAction::CloneAndMutate {
            base_table_name,
            target_table_name,
            replaced_paths,
            changed_payload: changed_rows,
        } => {
            let parquet_stats = write_mutated_table(
                service,
                base_table_name.as_str(),
                target_table_name.as_str(),
                replaced_paths,
                changed_rows,
            )
            .await?;
            finalize_repo_entity_publication(
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

async fn refresh_repo_entity_publication(
    service: &SearchPlaneService,
    repo_id: &str,
    table_name: &str,
    source_revision: Option<&str>,
    file_fingerprints: &BTreeMap<String, crate::search::SearchFileFingerprint>,
) -> Result<(), VectorStoreError> {
    let parquet_stats = inspect_repo_entity_parquet(service, table_name).await?;
    service
        .record_repo_publication_input_with_storage_format(
            SearchCorpusKind::RepoEntity,
            repo_id,
            SearchRepoPublicationInput {
                table_name: table_name.to_string(),
                schema_version: SearchCorpusKind::RepoEntity.schema_version(),
                source_revision: source_revision.map(str::to_string),
                table_version_id: parquet_stats.table_version_id,
                row_count: parquet_stats.row_count,
                fragment_count: parquet_stats.fragment_count,
                published_at: parquet_stats.published_at,
            },
            SearchPublicationStorageFormat::Parquet,
        )
        .await;
    set_repo_entity_file_fingerprints(service, repo_id, file_fingerprints).await;
    Ok(())
}

async fn set_repo_entity_file_fingerprints(
    service: &SearchPlaneService,
    repo_id: &str,
    file_fingerprints: &BTreeMap<String, crate::search::SearchFileFingerprint>,
) {
    service
        .set_file_fingerprints(
            SearchPlaneFileFingerprintScope::repo_corpus(SearchCorpusKind::RepoEntity, repo_id),
            file_fingerprints,
        )
        .await;
}

async fn finalize_repo_entity_publication(
    service: &SearchPlaneService,
    repo_id: &str,
    table_name: &str,
    source_revision: Option<&str>,
    parquet_stats: crate::search::repo_publication_parquet::ParquetPublicationStats,
    file_fingerprints: &BTreeMap<String, crate::search::SearchFileFingerprint>,
) -> Result<(), VectorStoreError> {
    let mut prewarm_columns = projected_columns().to_vec();
    prewarm_columns.push(hit_json_column());
    service
        .prewarm_repo_table(
            SearchCorpusKind::RepoEntity,
            repo_id,
            table_name,
            &prewarm_columns,
        )
        .await?;
    service
        .record_repo_publication_input_with_storage_format(
            SearchCorpusKind::RepoEntity,
            repo_id,
            SearchRepoPublicationInput {
                table_name: table_name.to_string(),
                schema_version: SearchCorpusKind::RepoEntity.schema_version(),
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
            SearchPlaneFileFingerprintScope::repo_corpus(SearchCorpusKind::RepoEntity, repo_id),
            file_fingerprints,
        )
        .await;
    Ok(())
}
