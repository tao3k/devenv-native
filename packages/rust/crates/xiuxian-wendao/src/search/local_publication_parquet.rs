use std::collections::BTreeSet;
use std::sync::Arc;

use arrow::array::{Array, BooleanArray, LargeStringArray, StringArray, StringViewArray};
use arrow::compute::filter_record_batch;
use xiuxian_db_store::{
    EngineRecordBatch, LanceRecordBatch, LanceSchema, SearchEngineContext, VectorStoreError,
    lance_batches_to_engine_batches, write_engine_batches_to_parquet_file,
};

use crate::search::{SearchCorpusKind, SearchPlaneService};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LocalParquetPublicationStats {
    pub(crate) row_count: u64,
    pub(crate) fragment_count: u64,
}

pub(crate) struct LocalParquetRewriteRequest<'a> {
    pub(crate) corpus: SearchCorpusKind,
    pub(crate) base_table_name: Option<&'a str>,
    pub(crate) target_table_name: &'a str,
    pub(crate) path_column: &'a str,
    pub(crate) replaced_paths: &'a BTreeSet<String>,
    pub(crate) changed_batches: &'a [LanceRecordBatch],
    pub(crate) empty_schema: Option<Arc<LanceSchema>>,
}

pub(crate) async fn rewrite_local_publication_parquet(
    service: &SearchPlaneService,
    request: LocalParquetRewriteRequest<'_>,
) -> Result<LocalParquetPublicationStats, VectorStoreError> {
    let mut output_batches = if let Some(base_table_name) = request.base_table_name {
        load_local_publication_parquet_batches(service, request.corpus, base_table_name).await?
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

    output_batches.extend(lance_batches_to_engine_batches(request.changed_batches)?);

    let parquet_path = service.local_table_parquet_path(request.corpus, request.target_table_name);
    if output_batches.is_empty() {
        if let Some(schema) = request.empty_schema {
            write_empty_local_publication_parquet(parquet_path.as_path(), schema)?;
        } else {
            match std::fs::remove_file(parquet_path.as_path()) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(VectorStoreError::Io(error)),
            }
        }
        return Ok(LocalParquetPublicationStats {
            row_count: 0,
            fragment_count: 0,
        });
    }

    write_engine_batches_to_parquet_file(parquet_path.as_path(), &output_batches)?;
    Ok(stats_from_batches(&output_batches))
}

fn write_empty_local_publication_parquet(
    output_path: &std::path::Path,
    schema: Arc<LanceSchema>,
) -> Result<(), VectorStoreError> {
    let empty_batch = LanceRecordBatch::new_empty(schema);
    let engine_batches = lance_batches_to_engine_batches(&[empty_batch])?;
    write_engine_batches_to_parquet_file(output_path, &engine_batches)
}

async fn load_local_publication_parquet_batches(
    service: &SearchPlaneService,
    corpus: SearchCorpusKind,
    table_name: &str,
) -> Result<Vec<EngineRecordBatch>, VectorStoreError> {
    let parquet_path = service.local_table_parquet_path(corpus, table_name);
    let engine = SearchEngineContext::new();
    engine
        .register_parquet_table("local_publication_source", parquet_path.as_path(), &[])
        .await?;
    let dataframe = engine.table("local_publication_source").await?;
    engine.collect_dataframe(dataframe).await
}

fn filter_batch_excluding_paths(
    batch: &EngineRecordBatch,
    path_column: &str,
    replaced_paths: &BTreeSet<String>,
) -> Result<Option<EngineRecordBatch>, VectorStoreError> {
    let path_index = batch.schema().index_of(path_column).map_err(|error| {
        VectorStoreError::General(format!(
            "missing local publication path column `{path_column}` in parquet batch: {error}"
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
                        "failed to decode Utf8 local publication path column `{path_column}`"
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
                        "failed to decode LargeUtf8 local publication path column `{path_column}`"
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
                        "failed to decode Utf8View local publication path column `{path_column}`"
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
                "unsupported local publication path column type for `{path_column}`: {other:?}"
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

fn stats_from_batches(batches: &[EngineRecordBatch]) -> LocalParquetPublicationStats {
    let row_count = batches
        .iter()
        .map(|batch| u64::try_from(batch.num_rows()).unwrap_or(u64::MAX))
        .fold(0_u64, u64::saturating_add);
    let fragment_count = u64::try_from(batches.len()).unwrap_or(u64::MAX);
    LocalParquetPublicationStats {
        row_count,
        fragment_count,
    }
}
