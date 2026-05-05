use std::collections::BTreeSet;
use std::fs::File;
use std::time::Instant;

use arrow::array::{Array, BooleanArray, LargeStringArray, StringArray, StringViewArray};
use arrow::compute::{cast, filter_record_batch};
use chrono::Utc;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use serde::{Deserialize, Serialize};
use xiuxian_db_store::VectorStoreError;
use xiuxian_db_store::{
    EngineRecordBatch, LanceRecordBatch, lance_batches_to_engine_batches,
    write_engine_batches_to_parquet_file,
};

use crate::repo_index::RepoCodeDocument;
use crate::search::repo_content_chunk::build::partitions::{
    repo_content_chunk_partition_count_for_document_count,
    repo_content_chunk_partition_id_for_count, repo_content_chunk_partition_id_for_path,
};
use crate::search::repo_content_chunk::build::types::RepoContentChunkMutationWriteProfile;
use crate::search::repo_content_chunk::schema::{
    path_column, repo_content_chunk_batches, repo_content_chunk_engine_schema,
    repo_content_chunk_schema, rows_from_documents,
};
use crate::search::repo_publication_parquet::{
    ParquetPublicationStats, inspect_repo_publication_parquet,
    parquet_publication_stats_from_counts,
};
use crate::search::{SearchCorpusKind, SearchFileFingerprint, SearchPlaneService};

const REPO_CONTENT_CHUNK_STATS_FILE_NAME: &str = "_stats.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RepoContentChunkPartitionStats {
    row_count: u64,
    fragment_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RepoContentChunkPublicationStatsSnapshot {
    published_at: String,
    partitions: std::collections::BTreeMap<String, RepoContentChunkPartitionStats>,
}

impl RepoContentChunkPublicationStatsSnapshot {
    fn to_parquet_stats(&self, table_name: &str) -> ParquetPublicationStats {
        parquet_publication_stats_from_counts(
            table_name,
            self.partitions
                .values()
                .map(|stats| stats.row_count)
                .fold(0_u64, u64::saturating_add),
            self.partitions
                .values()
                .map(|stats| stats.fragment_count)
                .fold(0_u64, u64::saturating_add),
            self.published_at.clone(),
        )
    }
}

pub(crate) async fn write_replaced_table(
    service: &SearchPlaneService,
    table_name: &str,
    documents: &[RepoCodeDocument],
) -> Result<ParquetPublicationStats, VectorStoreError> {
    let rows = rows_from_documents(documents);
    let changed_batches = repo_content_chunk_batches(&rows)?;
    let output_batches = lance_batches_to_engine_batches(changed_batches.as_slice());
    write_partitioned_repo_content_output(
        service,
        table_name,
        &output_batches,
        repo_content_chunk_partition_count_for_document_count(documents.len()),
    )
}

pub(crate) async fn write_mutated_table(
    service: &SearchPlaneService,
    base_table_name: &str,
    target_table_name: &str,
    replaced_paths: &BTreeSet<String>,
    changed_documents: &[RepoCodeDocument],
    file_fingerprints: &std::collections::BTreeMap<String, SearchFileFingerprint>,
    previous_fingerprints: &std::collections::BTreeMap<String, SearchFileFingerprint>,
) -> Result<ParquetPublicationStats, VectorStoreError> {
    Ok(write_mutated_table_profiled(
        service,
        base_table_name,
        target_table_name,
        replaced_paths,
        changed_documents,
        file_fingerprints,
        previous_fingerprints,
    )
    .await?
    .0)
}

pub(crate) async fn write_mutated_table_profiled(
    service: &SearchPlaneService,
    base_table_name: &str,
    target_table_name: &str,
    replaced_paths: &BTreeSet<String>,
    changed_documents: &[RepoCodeDocument],
    file_fingerprints: &std::collections::BTreeMap<String, SearchFileFingerprint>,
    previous_fingerprints: &std::collections::BTreeMap<String, SearchFileFingerprint>,
) -> Result<
    (
        ParquetPublicationStats,
        RepoContentChunkMutationWriteProfile,
    ),
    VectorStoreError,
> {
    let mut profile = RepoContentChunkMutationWriteProfile::default();
    let base_path =
        service.repo_publication_parquet_path(SearchCorpusKind::RepoContentChunk, base_table_name);
    if !base_path.is_dir() {
        return Ok((
            rewrite_legacy_repo_content_publication_as_partitioned(
                service,
                base_table_name,
                target_table_name,
                replaced_paths,
                changed_documents,
            )?,
            profile,
        ));
    }

    let target_root = service
        .repo_publication_parquet_path(SearchCorpusKind::RepoContentChunk, target_table_name);
    remove_repo_content_output(target_root.as_path())?;
    std::fs::create_dir_all(target_root.as_path())?;
    let previous_stats_snapshot = read_repo_content_stats_snapshot(base_path.as_path())?;

    let partitioned_changed_documents =
        partition_changed_documents(changed_documents, file_fingerprints);
    let partitioned_replaced_paths = partition_replaced_paths(
        replaced_paths,
        file_fingerprints,
        previous_fingerprints,
        repo_content_chunk_partition_count_for_document_count(file_fingerprints.len()),
    );
    let touched_partitions = partitioned_changed_documents
        .keys()
        .chain(partitioned_replaced_paths.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    profile.touched_partition_count = touched_partitions.len();

    let mut wrote_partition = false;
    let copy_started = Instant::now();
    profile.untouched_partition_count = copy_untouched_partitions(
        base_path.as_path(),
        target_root.as_path(),
        &touched_partitions,
        &mut wrote_partition,
    )?;
    profile.copy_untouched_elapsed = copy_started.elapsed();
    let mut next_partition_stats = previous_stats_snapshot
        .as_ref()
        .map(|snapshot| snapshot.partitions.clone());
    wrote_partition |= rewrite_touched_partitions(
        base_path.as_path(),
        target_root.as_path(),
        &touched_partitions,
        &partitioned_replaced_paths,
        &partitioned_changed_documents,
        &mut next_partition_stats,
        &mut profile,
    )?;

    if !wrote_partition {
        let empty_partition_stats =
            write_empty_partitioned_repo_content_output(target_root.as_path())?;
        let snapshot = RepoContentChunkPublicationStatsSnapshot {
            published_at: Utc::now().to_rfc3339(),
            partitions: std::collections::BTreeMap::from([(
                "00".to_string(),
                empty_partition_stats,
            )]),
        };
        let snapshot_started = Instant::now();
        write_repo_content_stats_snapshot(target_root.as_path(), &snapshot)?;
        profile.write_snapshot_elapsed += snapshot_started.elapsed();
        return Ok((snapshot.to_parquet_stats(target_table_name), profile));
    }

    if let Some(partitions) = next_partition_stats {
        let snapshot = RepoContentChunkPublicationStatsSnapshot {
            published_at: Utc::now().to_rfc3339(),
            partitions,
        };
        let snapshot_started = Instant::now();
        write_repo_content_stats_snapshot(target_root.as_path(), &snapshot)?;
        profile.write_snapshot_elapsed += snapshot_started.elapsed();
        return Ok((snapshot.to_parquet_stats(target_table_name), profile));
    }

    Ok((
        inspect_repo_content_chunk_parquet(service, target_table_name).await?,
        profile,
    ))
}

pub(crate) async fn inspect_repo_content_chunk_parquet(
    service: &SearchPlaneService,
    table_name: &str,
) -> Result<ParquetPublicationStats, VectorStoreError> {
    let publication_root =
        service.repo_publication_parquet_path(SearchCorpusKind::RepoContentChunk, table_name);
    if publication_root.is_dir()
        && let Some(snapshot) = read_repo_content_stats_snapshot(publication_root.as_path())?
    {
        return Ok(snapshot.to_parquet_stats(table_name));
    }
    inspect_repo_publication_parquet(service, SearchCorpusKind::RepoContentChunk, table_name).await
}

fn rewrite_touched_partitions(
    base_root: &std::path::Path,
    target_root: &std::path::Path,
    touched_partitions: &BTreeSet<String>,
    partitioned_replaced_paths: &std::collections::BTreeMap<String, BTreeSet<String>>,
    partitioned_changed_documents: &std::collections::BTreeMap<String, Vec<RepoCodeDocument>>,
    next_partition_stats: &mut Option<
        std::collections::BTreeMap<String, RepoContentChunkPartitionStats>,
    >,
    profile: &mut RepoContentChunkMutationWriteProfile,
) -> Result<bool, VectorStoreError> {
    let mut wrote_partition = false;
    for partition_id in touched_partitions {
        let load_started = Instant::now();
        let mut output_batches =
            load_partitioned_repo_content_batches(base_root, partition_id.as_str())?;
        profile.load_touched_elapsed += load_started.elapsed();
        if let Some(replaced_paths) = partitioned_replaced_paths.get(partition_id.as_str()) {
            let filter_started = Instant::now();
            let mut filtered_batches = Vec::with_capacity(output_batches.len());
            for batch in &output_batches {
                if let Some(filtered) =
                    filter_batch_excluding_paths(batch, path_column(), replaced_paths)?
                {
                    filtered_batches.push(filtered);
                }
            }
            output_batches = filtered_batches;
            profile.filter_replaced_elapsed += filter_started.elapsed();
        }
        if let Some(changed_documents) = partitioned_changed_documents.get(partition_id.as_str()) {
            let changed_started = Instant::now();
            let changed_rows = rows_from_documents(changed_documents.as_slice());
            let changed_batches = repo_content_chunk_batches(&changed_rows)?;
            output_batches.extend(lance_batches_to_engine_batches(changed_batches.as_slice()));
            profile.changed_payload_elapsed += changed_started.elapsed();
        }
        if output_batches.is_empty() {
            if let Some(partition_stats) = next_partition_stats.as_mut() {
                partition_stats.remove(partition_id.as_str());
            }
            continue;
        }
        let write_started = Instant::now();
        let partition_stats = write_normalized_repo_content_batches(
            repo_content_chunk_partition_path(target_root, partition_id.as_str()).as_path(),
            &output_batches,
        )?;
        profile.write_touched_elapsed += write_started.elapsed();
        if let Some(next_partition_stats) = next_partition_stats.as_mut() {
            next_partition_stats.insert(partition_id.clone(), partition_stats);
        }
        wrote_partition = true;
    }
    Ok(wrote_partition)
}

fn rewrite_legacy_repo_content_publication_as_partitioned(
    service: &SearchPlaneService,
    base_table_name: &str,
    target_table_name: &str,
    replaced_paths: &BTreeSet<String>,
    changed_documents: &[RepoCodeDocument],
) -> Result<ParquetPublicationStats, VectorStoreError> {
    let changed_rows = rows_from_documents(changed_documents);
    let changed_batches = repo_content_chunk_batches(&changed_rows)?;
    let mut output_batches = load_repo_content_batches_from_path(
        service
            .repo_publication_parquet_path(SearchCorpusKind::RepoContentChunk, base_table_name)
            .as_path(),
    )?;
    if !replaced_paths.is_empty() {
        let mut filtered_batches = Vec::with_capacity(output_batches.len());
        for batch in &output_batches {
            if let Some(filtered) =
                filter_batch_excluding_paths(batch, path_column(), replaced_paths)?
            {
                filtered_batches.push(filtered);
            }
        }
        output_batches = filtered_batches;
    }
    output_batches.extend(lance_batches_to_engine_batches(changed_batches.as_slice()));
    write_partitioned_repo_content_output(
        service,
        target_table_name,
        &output_batches,
        repo_content_chunk_partition_count_for_document_count(count_unique_repo_content_paths(
            &output_batches,
        )),
    )
}

fn write_partitioned_repo_content_output(
    service: &SearchPlaneService,
    table_name: &str,
    output_batches: &[EngineRecordBatch],
    partition_count: usize,
) -> Result<ParquetPublicationStats, VectorStoreError> {
    let target_root =
        service.repo_publication_parquet_path(SearchCorpusKind::RepoContentChunk, table_name);
    remove_repo_content_output(target_root.as_path())?;
    std::fs::create_dir_all(target_root.as_path())?;
    let partitioned_batches = partition_repo_content_batches(output_batches, partition_count)?;
    if partitioned_batches.is_empty() {
        let empty_partition_stats =
            write_empty_partitioned_repo_content_output(target_root.as_path())?;
        let snapshot = RepoContentChunkPublicationStatsSnapshot {
            published_at: Utc::now().to_rfc3339(),
            partitions: std::collections::BTreeMap::from([(
                "00".to_string(),
                empty_partition_stats,
            )]),
        };
        write_repo_content_stats_snapshot(target_root.as_path(), &snapshot)?;
        return Ok(snapshot.to_parquet_stats(table_name));
    }
    let mut partition_stats = std::collections::BTreeMap::new();
    for (partition_id, batches) in partitioned_batches {
        let batch_stats = write_normalized_repo_content_batches(
            repo_content_chunk_partition_path(target_root.as_path(), partition_id.as_str())
                .as_path(),
            &batches,
        )?;
        partition_stats.insert(partition_id, batch_stats);
    }
    let snapshot = RepoContentChunkPublicationStatsSnapshot {
        published_at: Utc::now().to_rfc3339(),
        partitions: partition_stats,
    };
    write_repo_content_stats_snapshot(target_root.as_path(), &snapshot)?;
    Ok(snapshot.to_parquet_stats(table_name))
}

fn partition_changed_documents(
    changed_documents: &[RepoCodeDocument],
    file_fingerprints: &std::collections::BTreeMap<String, SearchFileFingerprint>,
) -> std::collections::BTreeMap<String, Vec<RepoCodeDocument>> {
    let mut partitioned = std::collections::BTreeMap::<String, Vec<RepoCodeDocument>>::new();
    let partition_count =
        repo_content_chunk_partition_count_for_document_count(file_fingerprints.len());
    for document in changed_documents {
        let partition_id = repo_content_chunk_partition_id_for_path(
            document.path.as_str(),
            file_fingerprints,
            partition_count,
        );
        partitioned
            .entry(partition_id)
            .or_default()
            .push(document.clone());
    }
    partitioned
}

fn partition_replaced_paths(
    replaced_paths: &BTreeSet<String>,
    file_fingerprints: &std::collections::BTreeMap<String, SearchFileFingerprint>,
    previous_fingerprints: &std::collections::BTreeMap<String, SearchFileFingerprint>,
    fallback_partition_count: usize,
) -> std::collections::BTreeMap<String, BTreeSet<String>> {
    let mut partitioned = std::collections::BTreeMap::<String, BTreeSet<String>>::new();
    for path in replaced_paths {
        let partition_id = previous_fingerprints
            .get(path.as_str())
            .and_then(|fingerprint| fingerprint.partition_id.clone())
            .or_else(|| {
                file_fingerprints
                    .get(path.as_str())
                    .and_then(|fingerprint| fingerprint.partition_id.clone())
            })
            .unwrap_or_else(|| {
                repo_content_chunk_partition_id_for_count(path.as_str(), fallback_partition_count)
            });
        partitioned
            .entry(partition_id)
            .or_default()
            .insert(path.clone());
    }
    partitioned
}

fn partition_repo_content_batches(
    batches: &[EngineRecordBatch],
    partition_count: usize,
) -> Result<std::collections::BTreeMap<String, Vec<EngineRecordBatch>>, VectorStoreError> {
    let mut partitioned = std::collections::BTreeMap::<String, Vec<EngineRecordBatch>>::new();
    for batch in batches {
        let path_index = batch.schema().index_of(path_column()).map_err(|error| {
            VectorStoreError::General(format!(
                "missing repo-content path column `{}` in parquet batch: {error}",
                path_column()
            ))
        })?;
        let paths = batch.column(path_index);
        let path_values = decode_path_values(paths, path_column())?;
        let mut batch_partitions = BTreeSet::new();
        for path in &path_values {
            if let Some(path) = path.as_deref() {
                batch_partitions.insert(repo_content_chunk_partition_id_for_count(
                    path,
                    partition_count,
                ));
            }
        }
        for partition_id in batch_partitions {
            let keep_mask = BooleanArray::from(
                path_values
                    .iter()
                    .map(|path| {
                        path.as_ref().is_none_or(|path| {
                            repo_content_chunk_partition_id_for_count(path, partition_count)
                                == partition_id
                        })
                    })
                    .collect::<Vec<_>>(),
            );
            let filtered = filter_record_batch(batch, &keep_mask)?;
            if filtered.num_rows() == 0 {
                continue;
            }
            partitioned.entry(partition_id).or_default().push(filtered);
        }
    }
    Ok(partitioned)
}

fn count_unique_repo_content_paths(batches: &[EngineRecordBatch]) -> usize {
    let mut unique_paths = BTreeSet::<String>::new();
    for batch in batches {
        let Ok(path_index) = batch.schema().index_of(path_column()) else {
            continue;
        };
        let Ok(path_values) = decode_path_values(batch.column(path_index), path_column()) else {
            continue;
        };
        for path in path_values.into_iter().flatten() {
            unique_paths.insert(path);
        }
    }
    unique_paths.len()
}

fn decode_path_values(
    path_values: &arrow::array::ArrayRef,
    path_column_name: &str,
) -> Result<Vec<Option<String>>, VectorStoreError> {
    match path_values.data_type() {
        arrow::datatypes::DataType::Utf8 => {
            let strings = path_values
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| {
                    VectorStoreError::General(format!(
                        "failed to decode Utf8 repo-content path column `{path_column_name}`"
                    ))
                })?;
            Ok((0..strings.len())
                .map(|row| (!strings.is_null(row)).then_some(strings.value(row).to_string()))
                .collect())
        }
        arrow::datatypes::DataType::LargeUtf8 => {
            let strings = path_values
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .ok_or_else(|| {
                    VectorStoreError::General(format!(
                        "failed to decode LargeUtf8 repo-content path column `{path_column_name}`"
                    ))
                })?;
            Ok((0..strings.len())
                .map(|row| (!strings.is_null(row)).then_some(strings.value(row).to_string()))
                .collect())
        }
        arrow::datatypes::DataType::Utf8View => {
            let strings = path_values
                .as_any()
                .downcast_ref::<StringViewArray>()
                .ok_or_else(|| {
                    VectorStoreError::General(format!(
                        "failed to decode Utf8View repo-content path column `{path_column_name}`"
                    ))
                })?;
            Ok((0..strings.len())
                .map(|row| (!strings.is_null(row)).then_some(strings.value(row).to_string()))
                .collect())
        }
        other => Err(VectorStoreError::General(format!(
            "unsupported repo-content path column type for `{path_column_name}`: {other:?}"
        ))),
    }
}

fn load_partitioned_repo_content_batches(
    publication_root: &std::path::Path,
    partition_id: &str,
) -> Result<Vec<EngineRecordBatch>, VectorStoreError> {
    let partition_path = repo_content_chunk_partition_path(publication_root, partition_id);
    if !partition_path.exists() {
        return Ok(Vec::new());
    }
    load_repo_content_batches_from_path(partition_path.as_path())
}

fn load_repo_content_batches_from_path(
    parquet_path: &std::path::Path,
) -> Result<Vec<EngineRecordBatch>, VectorStoreError> {
    let parquet_file = File::open(parquet_path)?;
    let reader = ParquetRecordBatchReaderBuilder::try_new(parquet_file)?.build()?;
    let batches = reader.collect::<Result<Vec<_>, _>>()?;
    normalize_repo_content_batches(&batches)
}

fn normalize_repo_content_batches(
    batches: &[EngineRecordBatch],
) -> Result<Vec<EngineRecordBatch>, VectorStoreError> {
    let target_schema = repo_content_chunk_engine_schema();
    batches
        .iter()
        .map(|batch| normalize_repo_content_batch(batch, target_schema.clone()))
        .collect()
}

fn write_normalized_repo_content_batches(
    output_path: &std::path::Path,
    batches: &[EngineRecordBatch],
) -> Result<RepoContentChunkPartitionStats, VectorStoreError> {
    let normalized_batches = normalize_repo_content_batches(batches)?;
    let stats = repo_content_partition_stats_from_batches(&normalized_batches);
    write_engine_batches_to_parquet_file(output_path, &normalized_batches)?;
    Ok(stats)
}

fn normalize_repo_content_batch(
    batch: &EngineRecordBatch,
    target_schema: arrow::datatypes::SchemaRef,
) -> Result<EngineRecordBatch, VectorStoreError> {
    let columns = target_schema
        .fields()
        .iter()
        .map(|field| {
            let source_index = batch.schema().index_of(field.name()).map_err(|error| {
                VectorStoreError::General(format!(
                    "missing repo-content column `{}` while normalizing parquet batch: {error}",
                    field.name()
                ))
            })?;
            let source_column = batch.column(source_index);
            if source_column.data_type() == field.data_type() {
                Ok(source_column.clone())
            } else {
                cast(source_column.as_ref(), field.data_type()).map_err(VectorStoreError::Arrow)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    EngineRecordBatch::try_new(target_schema, columns).map_err(VectorStoreError::Arrow)
}

fn copy_untouched_partitions(
    base_root: &std::path::Path,
    target_root: &std::path::Path,
    touched_partitions: &BTreeSet<String>,
    wrote_partition: &mut bool,
) -> Result<usize, VectorStoreError> {
    if !base_root.is_dir() {
        return Ok(0);
    }
    let mut copied_partition_count = 0;
    for entry in std::fs::read_dir(base_root)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        let Some(partition_id) = repo_content_chunk_partition_id_from_file_name(file_name.as_ref())
        else {
            continue;
        };
        if touched_partitions.contains(partition_id.as_str()) {
            continue;
        }
        let target_path = target_root.join(file_name.as_ref());
        materialize_untouched_partition_file(entry.path().as_path(), target_path.as_path())?;
        *wrote_partition = true;
        copied_partition_count += 1;
    }
    Ok(copied_partition_count)
}

fn materialize_untouched_partition_file(
    source_path: &std::path::Path,
    target_path: &std::path::Path,
) -> Result<(), VectorStoreError> {
    if let Ok(()) = std::fs::hard_link(source_path, target_path) {
        Ok(())
    } else {
        std::fs::copy(source_path, target_path)?;
        Ok(())
    }
}

fn repo_content_chunk_partition_path(
    publication_root: &std::path::Path,
    partition_id: &str,
) -> std::path::PathBuf {
    publication_root.join(format!("part_{partition_id}.parquet"))
}

fn repo_content_chunk_partition_id_from_file_name(file_name: &str) -> Option<String> {
    file_name
        .strip_prefix("part_")
        .and_then(|name| name.strip_suffix(".parquet"))
        .map(str::to_string)
}

fn remove_repo_content_output(target_root: &std::path::Path) -> Result<(), VectorStoreError> {
    if !target_root.exists() {
        return Ok(());
    }
    if target_root.is_dir() {
        std::fs::remove_dir_all(target_root)?;
    } else {
        std::fs::remove_file(target_root)?;
    }
    Ok(())
}

fn write_empty_partitioned_repo_content_output(
    target_root: &std::path::Path,
) -> Result<RepoContentChunkPartitionStats, VectorStoreError> {
    std::fs::create_dir_all(target_root)?;
    let empty_batch = LanceRecordBatch::new_empty(repo_content_chunk_schema());
    let empty_batches = lance_batches_to_engine_batches(&[empty_batch]);
    write_normalized_repo_content_batches(
        repo_content_chunk_partition_path(target_root, "00").as_path(),
        &empty_batches,
    )
}

fn repo_content_chunk_stats_snapshot_path(
    publication_root: &std::path::Path,
) -> std::path::PathBuf {
    publication_root.join(REPO_CONTENT_CHUNK_STATS_FILE_NAME)
}

fn write_repo_content_stats_snapshot(
    publication_root: &std::path::Path,
    snapshot: &RepoContentChunkPublicationStatsSnapshot,
) -> Result<(), VectorStoreError> {
    std::fs::write(
        repo_content_chunk_stats_snapshot_path(publication_root),
        serde_json::to_vec_pretty(snapshot)?,
    )?;
    Ok(())
}

fn read_repo_content_stats_snapshot(
    publication_root: &std::path::Path,
) -> Result<Option<RepoContentChunkPublicationStatsSnapshot>, VectorStoreError> {
    let snapshot_path = repo_content_chunk_stats_snapshot_path(publication_root);
    match std::fs::read(snapshot_path) {
        Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(VectorStoreError::Io(error)),
    }
}

fn repo_content_partition_stats_from_batches(
    batches: &[EngineRecordBatch],
) -> RepoContentChunkPartitionStats {
    RepoContentChunkPartitionStats {
        row_count: batches
            .iter()
            .map(|batch| u64::try_from(batch.num_rows()).unwrap_or(u64::MAX))
            .fold(0_u64, u64::saturating_add),
        fragment_count: u64::try_from(batches.len()).unwrap_or(u64::MAX),
    }
}

fn filter_batch_excluding_paths(
    batch: &EngineRecordBatch,
    path_column_name: &str,
    replaced_paths: &BTreeSet<String>,
) -> Result<Option<EngineRecordBatch>, VectorStoreError> {
    let path_index = batch.schema().index_of(path_column_name).map_err(|error| {
        VectorStoreError::General(format!(
            "missing repo-content path column `{path_column_name}` in parquet batch: {error}"
        ))
    })?;
    let path_values = batch.column(path_index);
    let keep_mask = BooleanArray::from(
        decode_path_values(path_values, path_column_name)?
            .into_iter()
            .map(|path| path.is_none_or(|path| !replaced_paths.contains(&path)))
            .collect::<Vec<_>>(),
    );
    let filtered = filter_record_batch(batch, &keep_mask)?;
    if filtered.num_rows() == 0 {
        Ok(None)
    } else {
        Ok(Some(filtered))
    }
}
