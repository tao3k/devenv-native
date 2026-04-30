use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use arrow::array::{Array, ArrayRef, StringArray, StringBuilder};
use arrow::record_batch::RecordBatch;

use super::DOCUMENT_RESOURCE_ARROW_CACHE_NAME;
use super::io::{read_arrow_file, write_arrow_file};

pub(in super::super) fn read_cached_document_batches(
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

pub(in super::super) fn mirror_artifact_to_output(
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
        mirror_artifact_entries(artifact_dir, output_dir)?;
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

fn mirror_artifact_entries(artifact_dir: &Path, output_dir: &Path) -> Result<(), String> {
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
    Ok(())
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
            Ok(rewrite_resource_path_column(
                array,
                artifact_prefix.as_ref(),
                output_prefix.as_ref(),
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    RecordBatch::try_new(batch.schema(), columns)
        .map_err(|error| format!("rewrite document extract resource paths: {error}"))
}

fn rewrite_resource_path_column(
    array: &StringArray,
    artifact_prefix: &str,
    output_prefix: &str,
) -> ArrayRef {
    let mut builder = StringBuilder::new();
    for row in 0..array.len() {
        if array.is_null(row) {
            builder.append_null();
            continue;
        }
        let value = array.value(row);
        if let Some(suffix) = value.strip_prefix(artifact_prefix) {
            builder.append_value(format!("{output_prefix}{suffix}"));
        } else {
            builder.append_value(value);
        }
    }
    Arc::new(builder.finish()) as ArrayRef
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
