use xiuxian_db_store::VectorStoreError;

use crate::search::attachment::build::{AttachmentBuildPlan, AttachmentWriteResult};
use crate::search::attachment::schema::{
    attachment_batches, attachment_schema, source_path_column,
};
use crate::search::local_publication_parquet::{
    LocalParquetRewriteRequest, rewrite_local_publication_parquet,
};
use crate::search::{SearchBuildLease, SearchCorpusKind, SearchPlaneService};

pub(crate) async fn write_attachment_epoch(
    service: &SearchPlaneService,
    lease: &SearchBuildLease,
    plan: &AttachmentBuildPlan,
) -> Result<AttachmentWriteResult, VectorStoreError> {
    let table_name = SearchPlaneService::table_name(SearchCorpusKind::Attachment, lease.epoch);
    let changed_batches = attachment_batches(plan.changed_hits.as_slice())?;
    let base_table_name = plan.base_epoch.and_then(|base_epoch| {
        let base_table_name =
            SearchPlaneService::table_name(SearchCorpusKind::Attachment, base_epoch);
        service
            .local_table_exists(SearchCorpusKind::Attachment, base_table_name.as_str())
            .then_some(base_table_name)
    });
    let parquet_stats = rewrite_local_publication_parquet(
        service,
        LocalParquetRewriteRequest {
            corpus: SearchCorpusKind::Attachment,
            base_table_name: base_table_name.as_deref(),
            target_table_name: table_name.as_str(),
            path_column: source_path_column(),
            replaced_paths: &plan.replaced_paths,
            changed_batches: &changed_batches,
            empty_schema: Some(attachment_schema()),
        },
    )
    .await?;
    Ok(AttachmentWriteResult {
        row_count: parquet_stats.row_count,
        fragment_count: parquet_stats.fragment_count,
    })
}
