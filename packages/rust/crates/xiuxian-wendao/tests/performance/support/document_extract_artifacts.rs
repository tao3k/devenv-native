use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::Path;
use std::sync::Arc;

use arrow::array::{Array, ArrayRef, Float64Array, Int32Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::ipc::reader::FileReader;
use arrow::ipc::writer::FileWriter;
use arrow::record_batch::RecordBatch;
use serde::Serialize;

const DOCUMENT_RESOURCES_ARROW_CACHE_NAME: &str = "_resources.arrow";
const DOCUMENT_STRUCTURE_ARROW_CACHE_NAME: &str = "_structure.arrow";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArtifactReport {
    pub(crate) source: String,
    pub(crate) output_dir: String,
    pub(crate) resources_arrow_exists: bool,
    pub(crate) resources_arrow_bytes: u64,
    pub(crate) resources_row_count: usize,
    pub(crate) resource_type_counts: BTreeMap<String, usize>,
    pub(crate) resource_status_counts: BTreeMap<String, usize>,
    pub(crate) structure_arrow_exists: bool,
    pub(crate) structure_arrow_bytes: u64,
    pub(crate) structure_row_count: usize,
    pub(crate) structure_block_type_counts: BTreeMap<String, usize>,
    pub(crate) structure_ocr_page_blocks: usize,
    pub(crate) structure_ocr_region_blocks: usize,
    pub(crate) structure_bbox_blocks: usize,
    pub(crate) structure_reading_order_sorted: Option<bool>,
    pub(crate) artifact_error: Option<String>,
}

pub(crate) fn inspect_artifacts<'a>(
    inputs: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Vec<ArtifactReport> {
    let mut unique_outputs = BTreeMap::new();
    for (source, output_dir) in inputs {
        unique_outputs
            .entry(output_dir.to_string())
            .or_insert_with(|| source.to_string());
    }
    unique_outputs
        .into_iter()
        .map(|(output_dir, source)| inspect_artifact_dir(source.as_str(), output_dir.as_str()))
        .collect()
}

fn inspect_artifact_dir(source: &str, output_dir: &str) -> ArtifactReport {
    let mut report = ArtifactReport {
        source: source.to_string(),
        output_dir: output_dir.to_string(),
        resources_arrow_exists: false,
        resources_arrow_bytes: 0,
        resources_row_count: 0,
        resource_type_counts: BTreeMap::new(),
        resource_status_counts: BTreeMap::new(),
        structure_arrow_exists: false,
        structure_arrow_bytes: 0,
        structure_row_count: 0,
        structure_block_type_counts: BTreeMap::new(),
        structure_ocr_page_blocks: 0,
        structure_ocr_region_blocks: 0,
        structure_bbox_blocks: 0,
        structure_reading_order_sorted: None,
        artifact_error: None,
    };
    if let Err(error) = populate_artifact_report(&mut report) {
        report.artifact_error = Some(error);
    }
    report
}

fn populate_artifact_report(report: &mut ArtifactReport) -> Result<(), String> {
    let output_dir = Path::new(&report.output_dir);
    let resources_path = output_dir.join(DOCUMENT_RESOURCES_ARROW_CACHE_NAME);
    if let Some(batches) = read_arrow_file_batches(resources_path.as_path())? {
        report.resources_arrow_exists = true;
        report.resources_arrow_bytes = file_len(resources_path.as_path())?;
        report.resources_row_count = batches.iter().map(RecordBatch::num_rows).sum();
        report.resource_type_counts = string_counts(&batches, "resourceType")?;
        report.resource_status_counts = string_counts(&batches, "status")?;
    }

    let structure_path = output_dir.join(DOCUMENT_STRUCTURE_ARROW_CACHE_NAME);
    if let Some(batches) = read_arrow_file_batches(structure_path.as_path())? {
        report.structure_arrow_exists = true;
        report.structure_arrow_bytes = file_len(structure_path.as_path())?;
        report.structure_row_count = batches.iter().map(RecordBatch::num_rows).sum();
        report.structure_block_type_counts = string_counts(&batches, "blockType")?;
        report.structure_ocr_page_blocks = report
            .structure_block_type_counts
            .get("ocr_page")
            .copied()
            .unwrap_or_default();
        report.structure_ocr_region_blocks = report
            .structure_block_type_counts
            .get("ocr_region")
            .copied()
            .unwrap_or_default();
        report.structure_bbox_blocks = bbox_block_count(&batches)?;
        report.structure_reading_order_sorted = Some(structure_reading_order_sorted(&batches)?);
    }
    Ok(())
}

fn read_arrow_file_batches(path: &Path) -> Result<Option<Vec<RecordBatch>>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let file = File::open(path)
        .map_err(|error| format!("open Arrow IPC file `{}`: {error}", path.display()))?;
    let reader = FileReader::try_new(file, None)
        .map_err(|error| format!("read Arrow IPC file `{}`: {error}", path.display()))?;
    let batches = reader
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("decode Arrow IPC file `{}`: {error}", path.display()))?;
    Ok(Some(batches))
}

fn file_len(path: &Path) -> Result<u64, String> {
    fs::metadata(path)
        .map(|metadata| metadata.len())
        .map_err(|error| format!("stat Arrow IPC file `{}`: {error}", path.display()))
}

fn string_counts(
    batches: &[RecordBatch],
    column_name: &str,
) -> Result<BTreeMap<String, usize>, String> {
    let mut counts = BTreeMap::new();
    for batch in batches {
        let Some(column) = batch.column_by_name(column_name) else {
            continue;
        };
        let Some(array) = column.as_any().downcast_ref::<StringArray>() else {
            return Err(format!(
                "document extract `{column_name}` column is not a string array"
            ));
        };
        for row in 0..array.len() {
            let value = if array.is_null(row) {
                ""
            } else {
                array.value(row)
            };
            *counts.entry(value.to_string()).or_insert(0) += 1;
        }
    }
    Ok(counts)
}

fn bbox_block_count(batches: &[RecordBatch]) -> Result<usize, String> {
    let mut count = 0;
    for batch in batches {
        let bbox_columns = [
            float64_column(batch, "bboxLeft")?,
            float64_column(batch, "bboxTop")?,
            float64_column(batch, "bboxRight")?,
            float64_column(batch, "bboxBottom")?,
        ];
        for row in 0..batch.num_rows() {
            if bbox_columns.iter().any(|column| !column.is_null(row)) {
                count += 1;
            }
        }
    }
    Ok(count)
}

fn structure_reading_order_sorted(batches: &[RecordBatch]) -> Result<bool, String> {
    let mut previous: Option<(i32, String, i32, String)> = None;
    for batch in batches {
        let page_index = int32_column(batch, "pageIndex")?;
        let reading_order_key = string_column(batch, "readingOrderKey")?;
        let block_index = int32_column(batch, "blockIndex")?;
        let block_id = string_column(batch, "blockId")?;
        for row in 0..batch.num_rows() {
            let current = (
                int32_value(page_index, row),
                string_value(reading_order_key, row),
                int32_value(block_index, row),
                string_value(block_id, row),
            );
            if let Some(previous) = &previous
                && current < *previous
            {
                return Ok(false);
            }
            previous = Some(current);
        }
    }
    Ok(true)
}

fn string_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a StringArray, String> {
    batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| format!("document structure `{name}` column is not utf8"))
}

fn int32_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a Int32Array, String> {
    batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<Int32Array>())
        .ok_or_else(|| format!("document structure `{name}` column is not int32"))
}

fn float64_column<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a Float64Array, String> {
    batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<Float64Array>())
        .ok_or_else(|| format!("document structure `{name}` column is not float64"))
}

fn string_value(column: &StringArray, row: usize) -> String {
    if column.is_null(row) {
        String::new()
    } else {
        column.value(row).to_string()
    }
}

fn int32_value(column: &Int32Array, row: usize) -> i32 {
    if column.is_null(row) {
        0
    } else {
        column.value(row)
    }
}

#[test]
fn artifact_report_reads_structure_sidecar_ordering() -> Result<(), String> {
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let output_dir = temp_dir.path();
    write_test_arrow_file(
        output_dir
            .join(DOCUMENT_RESOURCES_ARROW_CACHE_NAME)
            .as_path(),
        &resource_test_batch()?,
    )?;
    write_test_arrow_file(
        output_dir
            .join(DOCUMENT_STRUCTURE_ARROW_CACHE_NAME)
            .as_path(),
        &structure_test_batch()?,
    )?;

    let report = inspect_artifact_dir("fixture.pdf", output_dir.to_string_lossy().as_ref());

    assert_eq!(report.artifact_error, None);
    assert!(report.resources_arrow_exists);
    assert_eq!(report.resources_row_count, 2);
    assert_eq!(report.resource_type_counts.get("text_page"), Some(&1));
    assert!(report.structure_arrow_exists);
    assert_eq!(report.structure_row_count, 2);
    assert_eq!(report.structure_ocr_region_blocks, 1);
    assert_eq!(report.structure_bbox_blocks, 1);
    assert_eq!(report.structure_reading_order_sorted, Some(true));
    Ok(())
}

fn write_test_arrow_file(path: &Path, batch: &RecordBatch) -> Result<(), String> {
    let file = File::create(path).map_err(|error| error.to_string())?;
    let mut writer =
        FileWriter::try_new(file, batch.schema().as_ref()).map_err(|error| error.to_string())?;
    writer.write(batch).map_err(|error| error.to_string())?;
    writer.finish().map_err(|error| error.to_string())
}

fn resource_test_batch() -> Result<RecordBatch, String> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("resourceType", DataType::Utf8, true),
        Field::new("status", DataType::Utf8, true),
    ]));
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(vec!["text_page", "ocr_text"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["ok", "ok"])) as ArrayRef,
        ],
    )
    .map_err(|error| error.to_string())
}

fn structure_test_batch() -> Result<RecordBatch, String> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("pageIndex", DataType::Int32, true),
        Field::new("readingOrderKey", DataType::Utf8, true),
        Field::new("blockIndex", DataType::Int32, true),
        Field::new("blockId", DataType::Utf8, true),
        Field::new("blockType", DataType::Utf8, true),
        Field::new("bboxLeft", DataType::Float64, true),
        Field::new("bboxTop", DataType::Float64, true),
        Field::new("bboxRight", DataType::Float64, true),
        Field::new("bboxBottom", DataType::Float64, true),
    ]));
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int32Array::from(vec![0, 0])) as ArrayRef,
            Arc::new(StringArray::from(vec!["000000.000000", "000000.000001"])) as ArrayRef,
            Arc::new(Int32Array::from(vec![0, 1])) as ArrayRef,
            Arc::new(StringArray::from(vec!["native", "region"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["text_page", "ocr_region"])) as ArrayRef,
            Arc::new(Float64Array::from(vec![None, Some(10.0)])) as ArrayRef,
            Arc::new(Float64Array::from(vec![None, Some(20.0)])) as ArrayRef,
            Arc::new(Float64Array::from(vec![None, Some(110.0)])) as ArrayRef,
            Arc::new(Float64Array::from(vec![None, Some(220.0)])) as ArrayRef,
        ],
    )
    .map_err(|error| error.to_string())
}
