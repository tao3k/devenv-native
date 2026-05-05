use crate::search::knowledge_section::build::types::{
    KnowledgeSectionBuildPlan, KnowledgeSectionWriteResult,
};
use crate::search::knowledge_section::schema::{
    knowledge_section_batches, knowledge_section_schema, path_column,
};
use crate::search::local_publication_parquet::{
    LocalParquetRewriteRequest, rewrite_local_publication_parquet,
};
use crate::search::{SearchBuildLease, SearchCorpusKind, SearchPlaneService};
use xiuxian_db_store::VectorStoreError;

#[cfg(any(test, feature = "test-support"))]
use crate::search::BeginBuildDecision;
#[cfg(any(test, feature = "test-support"))]
use crate::search::contracts::SearchProjectConfig;
#[cfg(any(test, feature = "test-support"))]
use crate::search::knowledge_section::build::orchestration::plan_knowledge_section_build;
#[cfg(any(test, feature = "test-support"))]
use crate::search::knowledge_section::build::types::KnowledgeSectionBuildError;
#[cfg(any(test, feature = "test-support"))]
use crate::search::knowledge_section::schema::projected_columns;
#[cfg(any(test, feature = "test-support"))]
use std::collections::BTreeMap;
#[cfg(any(test, feature = "test-support"))]
use std::path::Path;

pub(super) async fn write_knowledge_section_epoch(
    service: &SearchPlaneService,
    lease: &SearchBuildLease,
    plan: &KnowledgeSectionBuildPlan,
) -> Result<KnowledgeSectionWriteResult, VectorStoreError> {
    let table_name =
        SearchPlaneService::table_name(SearchCorpusKind::KnowledgeSection, lease.epoch);
    let changed_batches = knowledge_section_batches(plan.changed_rows.as_slice())?;
    let base_table_name = plan.base_epoch.and_then(|base_epoch| {
        let base_table_name =
            SearchPlaneService::table_name(SearchCorpusKind::KnowledgeSection, base_epoch);
        service
            .local_table_exists(SearchCorpusKind::KnowledgeSection, base_table_name.as_str())
            .then_some(base_table_name)
    });
    let parquet_stats = rewrite_local_publication_parquet(
        service,
        LocalParquetRewriteRequest {
            corpus: SearchCorpusKind::KnowledgeSection,
            base_table_name: base_table_name.as_deref(),
            target_table_name: table_name.as_str(),
            path_column: path_column(),
            replaced_paths: &plan.replaced_paths,
            changed_batches: &changed_batches,
            empty_schema: Some(knowledge_section_schema()),
        },
    )
    .await?;
    Ok(KnowledgeSectionWriteResult {
        row_count: parquet_stats.row_count,
        fragment_count: parquet_stats.fragment_count,
    })
}

#[cfg(any(test, feature = "test-support"))]
pub async fn publish_knowledge_sections_from_projects(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
    fingerprint: &str,
) -> Result<(), KnowledgeSectionBuildError> {
    let lease = match service.coordinator().begin_build(
        SearchCorpusKind::KnowledgeSection,
        fingerprint,
        SearchCorpusKind::KnowledgeSection.schema_version(),
    ) {
        BeginBuildDecision::Started(lease) => lease,
        BeginBuildDecision::AlreadyReady(_) | BeginBuildDecision::AlreadyIndexing(_) => {
            return Ok(());
        }
    };
    let plan = plan_knowledge_section_build(
        service,
        project_root,
        config_root,
        projects,
        None,
        &BTreeMap::new(),
    );
    match write_knowledge_section_epoch(service, &lease, &plan).await {
        Ok(write) => {
            let prewarm_columns = projected_columns();
            service
                .prewarm_epoch_table(lease.corpus, lease.epoch, &prewarm_columns)
                .await?;
            service
                .publish_ready_and_maintain(&lease, write.row_count, write.fragment_count)
                .await;
            Ok(())
        }
        Err(error) => {
            service.coordinator().fail_build(
                &lease,
                format!("knowledge section epoch write failed: {error}"),
            );
            Err(KnowledgeSectionBuildError::Storage(error))
        }
    }
}
