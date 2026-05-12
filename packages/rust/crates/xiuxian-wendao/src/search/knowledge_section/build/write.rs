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
