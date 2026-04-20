use xiuxian_db_store::VectorStoreError;

use crate::search::local_publication_parquet::{
    LocalParquetRewriteRequest, rewrite_local_publication_parquet,
};
use crate::search::reference_occurrence::build::{
    ReferenceOccurrenceBuildPlan, ReferenceOccurrenceWriteResult,
};
use crate::search::reference_occurrence::schema::{
    path_column, reference_occurrence_batches, reference_occurrence_schema,
};
use crate::search::{SearchBuildLease, SearchCorpusKind, SearchPlaneService};

pub(crate) async fn write_reference_occurrence_epoch(
    service: &SearchPlaneService,
    lease: &SearchBuildLease,
    plan: &ReferenceOccurrenceBuildPlan,
) -> Result<ReferenceOccurrenceWriteResult, VectorStoreError> {
    let table_name =
        SearchPlaneService::table_name(SearchCorpusKind::ReferenceOccurrence, lease.epoch);
    let changed_batches = reference_occurrence_batches(plan.changed_hits.as_slice())?;
    let base_table_name = plan.base_epoch.and_then(|base_epoch| {
        let base_table_name =
            SearchPlaneService::table_name(SearchCorpusKind::ReferenceOccurrence, base_epoch);
        service
            .local_table_exists(
                SearchCorpusKind::ReferenceOccurrence,
                base_table_name.as_str(),
            )
            .then_some(base_table_name)
    });
    let parquet_stats = rewrite_local_publication_parquet(
        service,
        LocalParquetRewriteRequest {
            corpus: SearchCorpusKind::ReferenceOccurrence,
            base_table_name: base_table_name.as_deref(),
            target_table_name: table_name.as_str(),
            path_column: path_column(),
            replaced_paths: &plan.replaced_paths,
            changed_batches: &changed_batches,
            empty_schema: Some(reference_occurrence_schema()),
        },
    )
    .await?;
    Ok(ReferenceOccurrenceWriteResult {
        row_count: parquet_stats.row_count,
        fragment_count: parquet_stats.fragment_count,
    })
}
