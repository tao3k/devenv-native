use std::collections::BTreeSet;
use std::sync::Arc;

use arrow::array::{Array, BooleanArray, LargeStringArray, StringArray, StringViewArray};
use arrow::compute::filter_record_batch;
use chrono::{DateTime, Utc};
use xiuxian_db_store::{
    EngineRecordBatch, LanceRecordBatch, LanceSchema, SearchEngineContext, VectorStoreError,
    lance_batches_to_engine_batches, write_engine_batches_to_parquet_file,
};

use crate::search::{SearchCorpusKind, SearchPlaneService};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParquetPublicationStats {
    pub(crate) table_version_id: u64,
    pub(crate) row_count: u64,
    pub(crate) fragment_count: u64,
    pub(crate) published_at: String,
}

pub(crate) struct RepoPublicationRewriteRequest<'a> {
    pub(crate) corpus: SearchCorpusKind,
    pub(crate) base_table_name: Option<&'a str>,
    pub(crate) target_table_name: &'a str,
    pub(crate) path_column: &'a str,
    pub(crate) replaced_paths: &'a BTreeSet<String>,
    pub(crate) changed_batches: &'a [LanceRecordBatch],
    pub(crate) empty_schema: Option<Arc<LanceSchema>>,
}

pub(crate) async fn rewrite_repo_publication_parquet(
    service: &SearchPlaneService,
    request: RepoPublicationRewriteRequest<'_>,
) -> Result<ParquetPublicationStats, VectorStoreError> {
    let mut output_batches = if let Some(base_table_name) = request.base_table_name {
        load_repo_publication_parquet_batches(service, request.corpus, base_table_name).await?
    } else {
        Vec::new()
    };

    if !request.replaced_paths.is_empty() {
        let mut filtered_batches = Vec::with_capacity(output_batches.len());
        for batch in &output_batches {
            if let Some(filtered) =
                filter_batch_excluding_paths(batch, request.path_column, request.replaced_paths)?
            {
                filtered_batches.push(filtered);
            }
        }
        output_batches = filtered_batches;
    }

    output_batches.extend(lance_batches_to_engine_batches(request.changed_batches));

    let parquet_path =
        service.repo_publication_parquet_path(request.corpus, request.target_table_name);
    if output_batches.is_empty() {
        if let Some(schema) = request.empty_schema {
            write_empty_repo_publication_parquet(parquet_path.as_path(), schema)?;
        } else {
            match std::fs::remove_file(parquet_path.as_path()) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(VectorStoreError::Io(error)),
            }
        }
        let published_at = Utc::now().to_rfc3339();
        return Ok(stats_from_batches(
            request.target_table_name,
            &[],
            published_at,
        ));
    }

    write_engine_batches_to_parquet_file(parquet_path.as_path(), &output_batches)?;
    let published_at = Utc::now().to_rfc3339();
    Ok(stats_from_batches(
        request.target_table_name,
        &output_batches,
        published_at,
    ))
}

fn write_empty_repo_publication_parquet(
    output_path: &std::path::Path,
    schema: Arc<LanceSchema>,
) -> Result<(), VectorStoreError> {
    let empty_batch = LanceRecordBatch::new_empty(schema);
    let engine_batches = lance_batches_to_engine_batches(&[empty_batch]);
    write_engine_batches_to_parquet_file(output_path, &engine_batches)
}

pub(crate) async fn inspect_repo_publication_parquet(
    service: &SearchPlaneService,
    corpus: SearchCorpusKind,
    table_name: &str,
) -> Result<ParquetPublicationStats, VectorStoreError> {
    let parquet_path = service.repo_publication_parquet_path(corpus, table_name);
    let published_at =
        DateTime::<Utc>::from(std::fs::metadata(parquet_path.as_path())?.modified()?).to_rfc3339();
    let batches = load_repo_publication_parquet_batches(service, corpus, table_name).await?;
    Ok(stats_from_batches(table_name, &batches, published_at))
}

async fn load_repo_publication_parquet_batches(
    service: &SearchPlaneService,
    corpus: SearchCorpusKind,
    table_name: &str,
) -> Result<Vec<EngineRecordBatch>, VectorStoreError> {
    let parquet_path = service.repo_publication_parquet_path(corpus, table_name);
    let engine = SearchEngineContext::new();
    engine
        .register_parquet_table("repo_publication_source", parquet_path.as_path(), &[])
        .await?;
    let dataframe = engine.table("repo_publication_source").await?;
    engine.collect_dataframe(dataframe).await
}

fn filter_batch_excluding_paths(
    batch: &EngineRecordBatch,
    path_column: &str,
    replaced_paths: &BTreeSet<String>,
) -> Result<Option<EngineRecordBatch>, VectorStoreError> {
    let path_index = batch.schema().index_of(path_column).map_err(|error| {
        VectorStoreError::General(format!(
            "missing repo publication path column `{path_column}` in parquet batch: {error}"
        ))
    })?;
    let path_values = batch.column(path_index);
    let keep_mask = match path_values.data_type() {
        arrow::datatypes::DataType::Utf8 => {
            let strings = path_values
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| {
                    VectorStoreError::General(format!(
                        "failed to decode Utf8 repo publication path column `{path_column}`"
                    ))
                })?;
            BooleanArray::from(
                (0..strings.len())
                    .map(|row| strings.is_null(row) || !replaced_paths.contains(strings.value(row)))
                    .collect::<Vec<_>>(),
            )
        }
        arrow::datatypes::DataType::LargeUtf8 => {
            let strings = path_values
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .ok_or_else(|| {
                    VectorStoreError::General(format!(
                        "failed to decode LargeUtf8 repo publication path column `{path_column}`"
                    ))
                })?;
            BooleanArray::from(
                (0..strings.len())
                    .map(|row| strings.is_null(row) || !replaced_paths.contains(strings.value(row)))
                    .collect::<Vec<_>>(),
            )
        }
        arrow::datatypes::DataType::Utf8View => {
            let strings = path_values
                .as_any()
                .downcast_ref::<StringViewArray>()
                .ok_or_else(|| {
                    VectorStoreError::General(format!(
                        "failed to decode Utf8View repo publication path column `{path_column}`"
                    ))
                })?;
            BooleanArray::from(
                (0..strings.len())
                    .map(|row| strings.is_null(row) || !replaced_paths.contains(strings.value(row)))
                    .collect::<Vec<_>>(),
            )
        }
        other => {
            return Err(VectorStoreError::General(format!(
                "unsupported repo publication path column type for `{path_column}`: {other:?}"
            )));
        }
    };
    let filtered = filter_record_batch(batch, &keep_mask)?;
    if filtered.num_rows() == 0 {
        Ok(None)
    } else {
        Ok(Some(filtered))
    }
}

fn stats_from_batches(
    table_name: &str,
    batches: &[EngineRecordBatch],
    published_at: String,
) -> ParquetPublicationStats {
    let row_count = batches
        .iter()
        .map(|batch| u64::try_from(batch.num_rows()).unwrap_or(u64::MAX))
        .fold(0_u64, u64::saturating_add);
    let fragment_count = u64::try_from(batches.len()).unwrap_or(u64::MAX);
    parquet_publication_stats_from_counts(table_name, row_count, fragment_count, published_at)
}

pub(crate) fn parquet_publication_stats_from_counts(
    table_name: &str,
    row_count: u64,
    fragment_count: u64,
    published_at: String,
) -> ParquetPublicationStats {
    let payload = format!("{table_name}|{published_at}|{row_count}|{fragment_count}");
    let hash = blake3::hash(payload.as_bytes());
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&hash.as_bytes()[..8]);
    ParquetPublicationStats {
        table_version_id: u64::from_be_bytes(bytes),
        row_count,
        fragment_count,
        published_at,
    }
}
