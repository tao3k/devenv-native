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
use xiuxian_wendao_attachments::pdf::structure::{
    DocumentStructureBlock, DocumentStructureParitySummary, validate_document_structure_parity,
};

const DOCUMENT_RESOURCES_ARROW_CACHE_NAME: &str = "_resources.arrow";
const DOCUMENT_STRUCTURE_ARROW_CACHE_NAME: &str = "_structure.arrow";
const DOCUMENT_METRICS_ARROW_CACHE_NAME: &str = "_metrics.arrow";

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
    pub(crate) structure_baseline_dir: Option<String>,
    pub(crate) structure_parity: Option<DocumentStructureParitySummary>,
    pub(crate) structure_parity_error: Option<String>,
    pub(crate) metrics_arrow_exists: bool,
    pub(crate) metrics_arrow_bytes: u64,
    pub(crate) metrics_row_count: usize,
    pub(crate) metrics_status_counts: BTreeMap<String, usize>,
    pub(crate) metrics_shard_type_counts: BTreeMap<String, usize>,
    pub(crate) metrics_ocr_profile_counts: BTreeMap<String, usize>,
    pub(crate) metrics_result_chars: usize,
    pub(crate) metrics_bbox_count: usize,
    pub(crate) metrics_rust_scheduler_elapsed_ms: f64,
    pub(crate) artifact_error: Option<String>,
}

pub(crate) fn inspect_artifacts<'a>(
    inputs: impl IntoIterator<Item = (&'a str, &'a str)>,
    structure_baseline_root: Option<&Path>,
) -> Vec<ArtifactReport> {
    let mut unique_outputs = BTreeMap::new();
    for (source, output_dir) in inputs {
        unique_outputs
            .entry(output_dir.to_string())
            .or_insert_with(|| source.to_string());
    }
    unique_outputs
        .into_iter()
        .map(|(output_dir, source)| {
            inspect_artifact_dir(
                source.as_str(),
                output_dir.as_str(),
                structure_baseline_root,
            )
        })
        .collect()
}

fn inspect_artifact_dir(
    source: &str,
    output_dir: &str,
    structure_baseline_root: Option<&Path>,
) -> ArtifactReport {
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
        structure_baseline_dir: None,
        structure_parity: None,
        structure_parity_error: None,
        metrics_arrow_exists: false,
        metrics_arrow_bytes: 0,
        metrics_row_count: 0,
        metrics_status_counts: BTreeMap::new(),
        metrics_shard_type_counts: BTreeMap::new(),
        metrics_ocr_profile_counts: BTreeMap::new(),
        metrics_result_chars: 0,
        metrics_bbox_count: 0,
        metrics_rust_scheduler_elapsed_ms: 0.0,
        artifact_error: None,
    };
    if let Err(error) = populate_artifact_report(&mut report, structure_baseline_root) {
        report.artifact_error = Some(error);
    }
    report
}

fn populate_artifact_report(
    report: &mut ArtifactReport,
    structure_baseline_root: Option<&Path>,
) -> Result<(), String> {
    let output_dir = std::path::PathBuf::from(&report.output_dir);
    let resources_path = output_dir.join(DOCUMENT_RESOURCES_ARROW_CACHE_NAME);
    if let Some(batches) = read_arrow_file_batches(resources_path.as_path())? {
        report.resources_arrow_exists = true;
        report.resources_arrow_bytes = file_len(resources_path.as_path())?;
        report.resources_row_count = batches.iter().map(RecordBatch::num_rows).sum();
        report.resource_type_counts = string_counts(&batches, "resourceType")?;
        report.resource_status_counts = string_counts(&batches, "status")?;
    }

    let structure_path = output_dir.join(DOCUMENT_STRUCTURE_ARROW_CACHE_NAME);
    let mut structure_batches = None;
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
        structure_batches = Some(batches);
    }
    populate_structure_parity(
        report,
        output_dir.as_path(),
        structure_batches.as_deref(),
        structure_baseline_root,
    )?;

    let metrics_path = output_dir.join(DOCUMENT_METRICS_ARROW_CACHE_NAME);
    if let Some(batches) = read_arrow_file_batches(metrics_path.as_path())? {
        report.metrics_arrow_exists = true;
        report.metrics_arrow_bytes = file_len(metrics_path.as_path())?;
        report.metrics_row_count = batches.iter().map(RecordBatch::num_rows).sum();
        report.metrics_status_counts = string_counts(&batches, "status")?;
        report.metrics_shard_type_counts = string_counts(&batches, "shardType")?;
        report.metrics_ocr_profile_counts = string_counts(&batches, "ocrProfile")?;
        report.metrics_result_chars = sum_int32_column_values(&batches, "resultChars")?;
        report.metrics_bbox_count = sum_int32_column_values(&batches, "bboxCount")?;
        report.metrics_rust_scheduler_elapsed_ms =
            max_float64_column_value(&batches, "rustSchedulerElapsedMs")?;
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

fn sum_int32_column_values(batches: &[RecordBatch], column_name: &str) -> Result<usize, String> {
    let mut total = 0usize;
    for batch in batches {
        let Some(column) = batch.column_by_name(column_name) else {
            continue;
        };
        let Some(array) = column.as_any().downcast_ref::<Int32Array>() else {
            return Err(format!(
                "document metrics `{column_name}` column is not int32"
            ));
        };
        for row in 0..array.len() {
            if !array.is_null(row) && array.value(row) > 0 {
                total =
                    total.saturating_add(usize::try_from(array.value(row)).unwrap_or(usize::MAX));
            }
        }
    }
    Ok(total)
}

fn max_float64_column_value(batches: &[RecordBatch], column_name: &str) -> Result<f64, String> {
    let mut max_value = 0.0f64;
    for batch in batches {
        let Some(column) = batch.column_by_name(column_name) else {
            continue;
        };
        let Some(array) = column.as_any().downcast_ref::<Float64Array>() else {
            return Err(format!(
                "document metrics `{column_name}` column is not float64"
            ));
        };
        for row in 0..array.len() {
            if !array.is_null(row) {
                max_value = max_value.max(array.value(row));
            }
        }
    }
    Ok(max_value)
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

fn populate_structure_parity(
    report: &mut ArtifactReport,
    output_dir: &Path,
    candidate_batches: Option<&[RecordBatch]>,
    baseline_root: Option<&Path>,
) -> Result<(), String> {
    let Some(baseline_root) = baseline_root else {
        return Ok(());
    };
    let baseline_dir = baseline_artifact_dir(baseline_root, output_dir)?;
    report.structure_baseline_dir = Some(baseline_dir.to_string_lossy().to_string());
    let baseline_path = baseline_dir.join(DOCUMENT_STRUCTURE_ARROW_CACHE_NAME);
    let Some(baseline_batches) = read_arrow_file_batches(baseline_path.as_path())? else {
        report.structure_parity_error = Some(format!(
            "missing baseline structure Arrow `{}`",
            baseline_path.display()
        ));
        return Ok(());
    };
    let Some(candidate_batches) = candidate_batches else {
        report.structure_parity_error =
            Some("candidate artifact has no structure sidecar".to_string());
        return Ok(());
    };
    let baseline = decode_structure_blocks(&baseline_batches)?;
    let candidate = decode_structure_blocks(candidate_batches)?;
    match validate_document_structure_parity(baseline.as_slice(), candidate.as_slice()) {
        Ok(summary) => report.structure_parity = Some(summary),
        Err(error) => report.structure_parity_error = Some(error),
    }
    Ok(())
}

fn baseline_artifact_dir(
    baseline_root: &Path,
    output_dir: &Path,
) -> Result<std::path::PathBuf, String> {
    let Some(name) = output_dir.file_name() else {
        return Err(format!(
            "cannot derive baseline artifact name from `{}`",
            output_dir.display()
        ));
    };
    Ok(baseline_root.join(name))
}

fn decode_structure_blocks(batches: &[RecordBatch]) -> Result<Vec<DocumentStructureBlock>, String> {
    let mut blocks = Vec::new();
    for batch in batches {
        let contract_version = string_column(batch, "contractVersion")?;
        let source_path = string_column(batch, "sourcePath")?;
        let source_content_hash = string_column(batch, "sourceContentHash")?;
        let block_id = string_column(batch, "blockId")?;
        let parent_block_id = string_column(batch, "parentBlockId")?;
        let page_index = int32_column(batch, "pageIndex")?;
        let block_index = int32_column(batch, "blockIndex")?;
        let reading_order_key = string_column(batch, "readingOrderKey")?;
        let block_type = string_column(batch, "blockType")?;
        let resource_element_id = string_column(batch, "resourceElementId")?;
        let content = string_column(batch, "content")?;
        let mime_type = string_column(batch, "mimeType")?;
        let status = string_column(batch, "status")?;
        let engine = string_column(batch, "engine")?;
        let confidence = float64_column(batch, "confidence")?;
        let bbox_left = float64_column(batch, "bboxLeft")?;
        let bbox_top = float64_column(batch, "bboxTop")?;
        let bbox_right = float64_column(batch, "bboxRight")?;
        let bbox_bottom = float64_column(batch, "bboxBottom")?;
        let provenance = string_column(batch, "provenance")?;
        for row in 0..batch.num_rows() {
            blocks.push(DocumentStructureBlock {
                contract_version: string_value(contract_version, row),
                source_path: string_value(source_path, row),
                source_content_hash: string_value(source_content_hash, row),
                block_id: string_value(block_id, row),
                parent_block_id: string_value(parent_block_id, row),
                page_index: int32_value(page_index, row),
                block_index: int32_value(block_index, row),
                reading_order_key: string_value(reading_order_key, row),
                block_type: string_value(block_type, row),
                resource_element_id: string_value(resource_element_id, row),
                content: string_value(content, row),
                mime_type: string_value(mime_type, row),
                status: string_value(status, row),
                engine: string_value(engine, row),
                confidence: optional_float64_value(confidence, row),
                bbox_left: optional_float64_value(bbox_left, row),
                bbox_top: optional_float64_value(bbox_top, row),
                bbox_right: optional_float64_value(bbox_right, row),
                bbox_bottom: optional_float64_value(bbox_bottom, row),
                provenance: string_value(provenance, row),
            });
        }
    }
    Ok(blocks)
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

fn optional_float64_value(column: &Float64Array, row: usize) -> Option<f64> {
    if column.is_null(row) {
        None
    } else {
        Some(column.value(row))
    }
}

#[test]
fn artifact_report_reads_structure_sidecar_ordering() -> Result<(), String> {
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let output_dir = temp_dir.path().join("outputs").join("fixture-a");
    let baseline_dir = temp_dir.path().join("baselines").join("fixture-a");
    fs::create_dir_all(output_dir.as_path()).map_err(|error| error.to_string())?;
    fs::create_dir_all(baseline_dir.as_path()).map_err(|error| error.to_string())?;
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
    write_test_arrow_file(
        baseline_dir
            .join(DOCUMENT_STRUCTURE_ARROW_CACHE_NAME)
            .as_path(),
        &structure_baseline_test_batch()?,
    )?;
    write_test_arrow_file(
        output_dir.join(DOCUMENT_METRICS_ARROW_CACHE_NAME).as_path(),
        &metrics_test_batch()?,
    )?;

    let baseline_root = temp_dir.path().join("baselines");
    let report = inspect_artifact_dir(
        "fixture.pdf",
        output_dir.to_string_lossy().as_ref(),
        Some(baseline_root.as_path()),
    );

    assert_eq!(report.artifact_error, None);
    assert!(report.resources_arrow_exists);
    assert_eq!(report.resources_row_count, 2);
    assert_eq!(report.resource_type_counts.get("text_page"), Some(&1));
    assert!(report.structure_arrow_exists);
    assert_eq!(report.structure_row_count, 2);
    assert_eq!(report.structure_ocr_region_blocks, 1);
    assert_eq!(report.structure_bbox_blocks, 1);
    assert_eq!(report.structure_reading_order_sorted, Some(true));
    assert!(report.metrics_arrow_exists);
    assert_eq!(report.metrics_row_count, 2);
    assert_eq!(report.metrics_status_counts.get("succeeded"), Some(&2));
    assert_eq!(report.metrics_shard_type_counts.get("region"), Some(&1));
    assert_eq!(report.metrics_ocr_profile_counts.get("docling"), Some(&2));
    assert_eq!(report.metrics_result_chars, 42);
    assert_eq!(report.metrics_bbox_count, 1);
    assert!((report.metrics_rust_scheduler_elapsed_ms - 7.5).abs() < f64::EPSILON);
    let parity = report
        .structure_parity
        .ok_or_else(|| "expected structure parity summary".to_string())?;
    assert_eq!(parity.baseline_block_count, 1);
    assert_eq!(parity.candidate_block_count, 2);
    assert_eq!(report.structure_parity_error, None);
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
        Field::new("contractVersion", DataType::Utf8, true),
        Field::new("sourcePath", DataType::Utf8, true),
        Field::new("sourceContentHash", DataType::Utf8, true),
        Field::new("blockId", DataType::Utf8, true),
        Field::new("parentBlockId", DataType::Utf8, true),
        Field::new("pageIndex", DataType::Int32, true),
        Field::new("blockIndex", DataType::Int32, true),
        Field::new("readingOrderKey", DataType::Utf8, true),
        Field::new("blockType", DataType::Utf8, true),
        Field::new("resourceElementId", DataType::Utf8, true),
        Field::new("content", DataType::Utf8, true),
        Field::new("mimeType", DataType::Utf8, true),
        Field::new("status", DataType::Utf8, true),
        Field::new("engine", DataType::Utf8, true),
        Field::new("confidence", DataType::Float64, true),
        Field::new("bboxLeft", DataType::Float64, true),
        Field::new("bboxTop", DataType::Float64, true),
        Field::new("bboxRight", DataType::Float64, true),
        Field::new("bboxBottom", DataType::Float64, true),
        Field::new("provenance", DataType::Utf8, true),
    ]));
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(vec![
                "xiuxian_wendao.document_structure.v1",
                "xiuxian_wendao.document_structure.v1",
            ])) as ArrayRef,
            Arc::new(StringArray::from(vec!["fixture.pdf", "fixture.pdf"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["hash", "hash"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["native", "region"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["", "native"])) as ArrayRef,
            Arc::new(Int32Array::from(vec![0, 0])) as ArrayRef,
            Arc::new(Int32Array::from(vec![0, 1])) as ArrayRef,
            Arc::new(StringArray::from(vec!["000000.000000", "000000.000001"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["text_page", "ocr_region"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["native", "region"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["native text", "recognized text"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["text/markdown", "text/markdown"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["ok", "succeeded"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["docling", "wendao-hybrid"])) as ArrayRef,
            Arc::new(Float64Array::from(vec![None, Some(0.99)])) as ArrayRef,
            Arc::new(Float64Array::from(vec![None, Some(10.0)])) as ArrayRef,
            Arc::new(Float64Array::from(vec![None, Some(20.0)])) as ArrayRef,
            Arc::new(Float64Array::from(vec![None, Some(110.0)])) as ArrayRef,
            Arc::new(Float64Array::from(vec![None, Some(220.0)])) as ArrayRef,
            Arc::new(StringArray::from(vec!["{}", "{}"])) as ArrayRef,
        ],
    )
    .map_err(|error| error.to_string())
}

fn structure_baseline_test_batch() -> Result<RecordBatch, String> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("contractVersion", DataType::Utf8, true),
        Field::new("sourcePath", DataType::Utf8, true),
        Field::new("sourceContentHash", DataType::Utf8, true),
        Field::new("blockId", DataType::Utf8, true),
        Field::new("parentBlockId", DataType::Utf8, true),
        Field::new("pageIndex", DataType::Int32, true),
        Field::new("blockIndex", DataType::Int32, true),
        Field::new("readingOrderKey", DataType::Utf8, true),
        Field::new("blockType", DataType::Utf8, true),
        Field::new("resourceElementId", DataType::Utf8, true),
        Field::new("content", DataType::Utf8, true),
        Field::new("mimeType", DataType::Utf8, true),
        Field::new("status", DataType::Utf8, true),
        Field::new("engine", DataType::Utf8, true),
        Field::new("confidence", DataType::Float64, true),
        Field::new("bboxLeft", DataType::Float64, true),
        Field::new("bboxTop", DataType::Float64, true),
        Field::new("bboxRight", DataType::Float64, true),
        Field::new("bboxBottom", DataType::Float64, true),
        Field::new("provenance", DataType::Utf8, true),
    ]));
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(vec![
                "xiuxian_wendao.document_structure.v1",
            ])) as ArrayRef,
            Arc::new(StringArray::from(vec!["fixture.pdf"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["hash"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["native"])) as ArrayRef,
            Arc::new(StringArray::from(vec![""])) as ArrayRef,
            Arc::new(Int32Array::from(vec![0])) as ArrayRef,
            Arc::new(Int32Array::from(vec![0])) as ArrayRef,
            Arc::new(StringArray::from(vec!["000000.000000"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["text_page"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["native"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["native"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["text/markdown"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["ok"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["docling"])) as ArrayRef,
            Arc::new(Float64Array::from(vec![None])) as ArrayRef,
            Arc::new(Float64Array::from(vec![None])) as ArrayRef,
            Arc::new(Float64Array::from(vec![None])) as ArrayRef,
            Arc::new(Float64Array::from(vec![None])) as ArrayRef,
            Arc::new(Float64Array::from(vec![None])) as ArrayRef,
            Arc::new(StringArray::from(vec!["{}"])) as ArrayRef,
        ],
    )
    .map_err(|error| error.to_string())
}

fn metrics_test_batch() -> Result<RecordBatch, String> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("status", DataType::Utf8, true),
        Field::new("shardType", DataType::Utf8, true),
        Field::new("ocrProfile", DataType::Utf8, true),
        Field::new("resultChars", DataType::Int32, true),
        Field::new("bboxCount", DataType::Int32, true),
        Field::new("rustSchedulerElapsedMs", DataType::Float64, true),
    ]));
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(vec!["succeeded", "succeeded"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["page", "region"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["docling", "docling"])) as ArrayRef,
            Arc::new(Int32Array::from(vec![21, 21])) as ArrayRef,
            Arc::new(Int32Array::from(vec![0, 1])) as ArrayRef,
            Arc::new(Float64Array::from(vec![Some(5.0), Some(7.5)])) as ArrayRef,
        ],
    )
    .map_err(|error| error.to_string())
}
