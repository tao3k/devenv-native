use std::collections::BTreeSet;

use xiuxian_db_store::VectorStoreError;

use crate::search::local_publication_parquet::{
    LocalParquetRewriteRequest, rewrite_local_publication_parquet,
};
use crate::search::local_symbol::build::{LocalSymbolBuildPlan, LocalSymbolWriteResult};
use crate::search::local_symbol::schema::{local_symbol_batches, local_symbol_schema, path_column};
use crate::search::{SearchBuildLease, SearchCorpusKind, SearchPlaneService};

pub(crate) async fn write_local_symbol_epoch(
    service: &SearchPlaneService,
    lease: &SearchBuildLease,
    plan: &LocalSymbolBuildPlan,
) -> Result<LocalSymbolWriteResult, VectorStoreError> {
    let mut row_count = 0_u64;
    let mut fragment_count = 0_u64;
    let mut wrote_epoch_tables = false;

    for (partition_id, partition_plan) in &plan.partitions {
        let table_name = SearchPlaneService::local_partition_table_name(
            SearchCorpusKind::LocalSymbol,
            lease.epoch,
            partition_id.as_str(),
        );
        let changed_batches = local_symbol_batches(partition_plan.changed_hits.as_slice())?;

        let base_table_name = plan.base_epoch.and_then(|base_epoch| {
            let base_table_name = SearchPlaneService::local_partition_table_name(
                SearchCorpusKind::LocalSymbol,
                base_epoch,
                partition_id.as_str(),
            );
            service
                .local_table_exists(SearchCorpusKind::LocalSymbol, base_table_name.as_str())
                .then_some(base_table_name)
        });
        let parquet_stats = rewrite_local_publication_parquet(
            service,
            LocalParquetRewriteRequest {
                corpus: SearchCorpusKind::LocalSymbol,
                base_table_name: base_table_name.as_deref(),
                target_table_name: table_name.as_str(),
                path_column: path_column(),
                replaced_paths: &partition_plan.replaced_paths,
                changed_batches: &changed_batches,
                empty_schema: Some(local_symbol_schema()),
            },
        )
        .await?;
        wrote_epoch_tables = true;
        if parquet_stats.row_count == 0 {
            continue;
        }
        row_count = row_count.saturating_add(parquet_stats.row_count);
        fragment_count = fragment_count.saturating_add(parquet_stats.fragment_count);
    }

    if !wrote_epoch_tables {
        let table_name = SearchPlaneService::table_name(SearchCorpusKind::LocalSymbol, lease.epoch);
        let empty_paths = BTreeSet::new();
        let empty_batches = [];
        let parquet_stats = rewrite_local_publication_parquet(
            service,
            LocalParquetRewriteRequest {
                corpus: SearchCorpusKind::LocalSymbol,
                base_table_name: None,
                target_table_name: table_name.as_str(),
                path_column: path_column(),
                replaced_paths: &empty_paths,
                changed_batches: &empty_batches,
                empty_schema: Some(local_symbol_schema()),
            },
        )
        .await?;
        row_count = parquet_stats.row_count;
        fragment_count = parquet_stats.fragment_count;
    }

    Ok(LocalSymbolWriteResult {
        row_count,
        fragment_count,
    })
}
