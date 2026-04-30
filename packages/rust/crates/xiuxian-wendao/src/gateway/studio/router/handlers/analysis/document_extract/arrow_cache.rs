use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use arrow::array::{Array, ArrayRef, Int32Array, Int64Array, StringArray, StringBuilder};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::ipc::reader::FileReader;
use arrow::ipc::writer::FileWriter;
use arrow::record_batch::RecordBatch;

use super::registry::DocumentExtractJobStatus;

pub(super) const DOCUMENT_RESOURCE_ARROW_CACHE_NAME: &str = "_resources.arrow";

pub(super) fn document_resource_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("sourcePath", DataType::Utf8, true),
        Field::new("resourceType", DataType::Utf8, true),
        Field::new("resourcePath", DataType::Utf8, true),
        Field::new("pageIndex", DataType::Int32, true),
        Field::new("caption", DataType::Utf8, true),
        Field::new("content", DataType::Utf8, true),
        Field::new("mimeType", DataType::Utf8, true),
        Field::new("status", DataType::Utf8, true),
        Field::new("elementId", DataType::Utf8, true),
    ]))
}

pub(super) fn document_extract_status_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("jobId", DataType::Utf8, true),
        Field::new("sourcePath", DataType::Utf8, true),
        Field::new("outputDir", DataType::Utf8, true),
        Field::new("contentHash", DataType::Utf8, true),
        Field::new("status", DataType::Utf8, true),
        Field::new("attemptCount", DataType::Int32, true),
        Field::new("createdAtMs", DataType::Int64, true),
        Field::new("startedAtMs", DataType::Int64, true),
        Field::new("finishedAtMs", DataType::Int64, true),
        Field::new("errorMessage", DataType::Utf8, true),
    ]))
}

pub(super) fn read_cached_document_batches(
    source_path: &Path,
    output_dir: &Path,
) -> Result<Option<Vec<RecordBatch>>, String> {
    let marker_path = output_dir.join("_complete.marker");
    let resources_path = output_dir.join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME);
    if !marker_path.exists() || !resources_path.exists() {
        return Ok(None);
    }
    let source_modified = source_path
        .metadata()
        .and_then(|metadata| metadata.modified())
        .map_err(|error| format!("read source metadata `{}`: {error}", source_path.display()))?;
    let marker_modified = marker_path
        .metadata()
        .and_then(|metadata| metadata.modified())
        .map_err(|error| format!("read marker metadata `{}`: {error}", marker_path.display()))?;
    if source_modified > marker_modified {
        return Ok(None);
    }
    read_arrow_file(resources_path.as_path()).map(Some)
}

pub(super) fn build_job_resource_batch(
    status: &DocumentExtractJobStatus,
) -> Result<RecordBatch, String> {
    RecordBatch::try_new(
        document_resource_schema(),
        vec![
            string_column([status.source_path.as_str()]),
            string_column(["job"]),
            string_column([status.output_dir.as_str()]),
            Arc::new(Int32Array::from(vec![0])) as ArrayRef,
            string_column(["document extraction job"]),
            string_column([status.status.as_str()]),
            string_column(["application/vnd.xiuxian.document-extract-job"]),
            string_column([status.status.as_str()]),
            string_column([status.job_id.as_str()]),
        ],
    )
    .map_err(|error| format!("build document extract job batch: {error}"))
}

pub(super) fn build_error_resource_batch(
    status: &DocumentExtractJobStatus,
) -> Result<RecordBatch, String> {
    RecordBatch::try_new(
        document_resource_schema(),
        vec![
            string_column([status.source_path.as_str()]),
            string_column(["error"]),
            string_column([status.output_dir.as_str()]),
            Arc::new(Int32Array::from(vec![0])) as ArrayRef,
            string_column(["document extraction job failed"]),
            string_column([status.error_message.as_str()]),
            string_column(["text/plain"]),
            string_column(["error"]),
            string_column([status.job_id.as_str()]),
        ],
    )
    .map_err(|error| format!("build document extract error batch: {error}"))
}

pub(super) fn build_status_batch(status: &DocumentExtractJobStatus) -> Result<RecordBatch, String> {
    RecordBatch::try_new(
        document_extract_status_schema(),
        vec![
            string_column([status.job_id.as_str()]),
            string_column([status.source_path.as_str()]),
            string_column([status.output_dir.as_str()]),
            string_column([status.content_hash.as_str()]),
            string_column([status.status.as_str()]),
            Arc::new(Int32Array::from(vec![status.attempt_count])) as ArrayRef,
            Arc::new(Int64Array::from(vec![status.created_at_ms])) as ArrayRef,
            Arc::new(Int64Array::from(vec![status.started_at_ms])) as ArrayRef,
            Arc::new(Int64Array::from(vec![status.finished_at_ms])) as ArrayRef,
            string_column([status.error_message.as_str()]),
        ],
    )
    .map_err(|error| format!("build document extract status batch: {error}"))
}

pub(super) fn mirror_artifact_to_output(
    artifact_dir: &Path,
    output_dir: &Path,
) -> Result<(), String> {
    fs::create_dir_all(output_dir).map_err(|error| {
        format!(
            "create document extract output directory `{}`: {error}",
            output_dir.display()
        )
    })?;
    if artifact_dir.canonicalize().ok() != output_dir.canonicalize().ok() {
        for entry in fs::read_dir(artifact_dir).map_err(|error| {
            format!(
                "read document extract artifact directory `{}`: {error}",
                artifact_dir.display()
            )
        })? {
            let entry = entry.map_err(|error| {
                format!(
                    "read document extract artifact entry `{}`: {error}",
                    artifact_dir.display()
                )
            })?;
            let source = entry.path();
            let target = output_dir.join(entry.file_name());
            if source.is_dir() {
                if target.exists() {
                    fs::remove_dir_all(target.as_path()).map_err(|error| {
                        format!(
                            "remove stale document extract output `{}`: {error}",
                            target.display()
                        )
                    })?;
                }
                copy_dir_all(source.as_path(), target.as_path())?;
            } else {
                fs::copy(source.as_path(), target.as_path()).map_err(|error| {
                    format!(
                        "copy document extract artifact `{}` to `{}`: {error}",
                        source.display(),
                        target.display()
                    )
                })?;
            }
        }
    }

    let resources_path = output_dir.join(DOCUMENT_RESOURCE_ARROW_CACHE_NAME);
    if resources_path.exists() {
        let batches = read_arrow_file(resources_path.as_path())?;
        let rewritten = batches
            .iter()
            .map(|batch| rewrite_resource_paths(batch, artifact_dir, output_dir))
            .collect::<Result<Vec<_>, _>>()?;
        write_arrow_file(resources_path.as_path(), &rewritten)?;
    }
    File::create(output_dir.join("_complete.marker"))
        .map_err(|error| format!("touch document extract complete marker: {error}"))?;
    Ok(())
}

pub(super) fn read_arrow_file(path: &Path) -> Result<Vec<RecordBatch>, String> {
    let file = File::open(path)
        .map_err(|error| format!("open Arrow IPC file `{}`: {error}", path.display()))?;
    let reader = FileReader::try_new(file, None)
        .map_err(|error| format!("decode Arrow IPC file `{}`: {error}", path.display()))?;
    let mut batches = Vec::new();
    for batch in reader {
        batches.push(
            batch.map_err(|error| format!("read Arrow IPC batch `{}`: {error}", path.display()))?,
        );
    }
    Ok(batches)
}

pub(super) fn write_arrow_file(path: &Path, batches: &[RecordBatch]) -> Result<(), String> {
    let Some(first) = batches.first() else {
        return Err(format!(
            "cannot write empty Arrow IPC file `{}`",
            path.display()
        ));
    };
    let file = File::create(path)
        .map_err(|error| format!("create Arrow IPC file `{}`: {error}", path.display()))?;
    let mut writer = FileWriter::try_new(file, first.schema().as_ref())
        .map_err(|error| format!("create Arrow IPC writer `{}`: {error}", path.display()))?;
    for batch in batches {
        writer
            .write(batch)
            .map_err(|error| format!("write Arrow IPC batch `{}`: {error}", path.display()))?;
    }
    writer
        .finish()
        .map_err(|error| format!("finish Arrow IPC file `{}`: {error}", path.display()))
}

fn rewrite_resource_paths(
    batch: &RecordBatch,
    artifact_dir: &Path,
    output_dir: &Path,
) -> Result<RecordBatch, String> {
    let artifact_prefix = artifact_dir.to_string_lossy();
    let output_prefix = output_dir.to_string_lossy();
    let columns = batch
        .schema()
        .fields()
        .iter()
        .enumerate()
        .map(|(index, field)| {
            if field.name() != "resourcePath" {
                return Ok(Arc::clone(batch.column(index)));
            }
            let array = batch
                .column(index)
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| "document extract resourcePath column is not utf8".to_string())?;
            let mut builder = StringBuilder::new();
            for row in 0..array.len() {
                if array.is_null(row) {
                    builder.append_null();
                    continue;
                }
                let value = array.value(row);
                if let Some(suffix) = value.strip_prefix(artifact_prefix.as_ref()) {
                    builder.append_value(format!("{output_prefix}{suffix}"));
                } else {
                    builder.append_value(value);
                }
            }
            Ok(Arc::new(builder.finish()) as ArrayRef)
        })
        .collect::<Result<Vec<_>, String>>()?;
    RecordBatch::try_new(batch.schema(), columns)
        .map_err(|error| format!("rewrite document extract resource paths: {error}"))
}

fn string_column<'a>(values: impl IntoIterator<Item = &'a str>) -> ArrayRef {
    Arc::new(StringArray::from_iter_values(values)) as ArrayRef
}

fn copy_dir_all(source: &Path, target: &Path) -> Result<(), String> {
    fs::create_dir_all(target)
        .map_err(|error| format!("create directory `{}`: {error}", target.display()))?;
    for entry in fs::read_dir(source)
        .map_err(|error| format!("read directory `{}`: {error}", source.display()))?
    {
        let entry = entry
            .map_err(|error| format!("read directory entry `{}`: {error}", source.display()))?;
        let target_path: PathBuf = target.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir_all(entry.path().as_path(), target_path.as_path())?;
        } else {
            fs::copy(entry.path(), target_path.as_path()).map_err(|error| {
                format!(
                    "copy file `{}` to `{}`: {error}",
                    entry.path().display(),
                    target_path.display()
                )
            })?;
        }
    }
    Ok(())
}
