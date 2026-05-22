use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::Path;
use std::sync::Arc;

use arrow::array::ArrayRef;
use arrow::array::{Array, Float64Array, Int32Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::ipc::reader::FileReader;
use arrow::ipc::writer::FileWriter;
use arrow::record_batch::RecordBatch;
use serde::Serialize;
use serde_json::Value;

#[cfg(feature = "document-extract-attachment-audit")]
use xiuxian_wendao_attachments::archive_audit::{
    ArchiveAttachmentAudit, audit_archive_attachment, is_supported_archive_path,
};
#[cfg(feature = "document-extract-attachment-audit")]
use xiuxian_wendao_attachments::image_audit::{
    AttachmentAudit, audit_image_attachment, is_supported_image_path,
};
#[cfg(feature = "document-extract-pdf-source-range")]
use xiuxian_wendao_attachments::pdf::structure::{
    DocumentStructureBlock, validate_document_structure_parity,
};

const DOCUMENT_RESOURCES_ARROW_CACHE_NAME: &str = "_resources.arrow";
const DOCUMENT_STRUCTURE_ARROW_CACHE_NAME: &str = "_structure.arrow";
const DOCUMENT_METRICS_ARROW_CACHE_NAME: &str = "_metrics.arrow";
const DOCUMENT_TIMING_ARROW_CACHE_NAME: &str = "_document_metrics.arrow";
const HYBRID_PAGE_OCR_FALLBACK_REPORT_NAME: &str = "_hybrid_page_ocr_fallback.json";
const HYBRID_PAGE_OCR_TIMING_REPORT_NAME: &str = "_hybrid_page_ocr_timing.json";

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
    pub(crate) audio_transcript_chars: usize,
    pub(crate) audio_transcript_timeline_marker_count: usize,
    pub(crate) audio_transcript_timeline_marked_rows: usize,
    pub(crate) structure_arrow_exists: bool,
    pub(crate) structure_arrow_bytes: u64,
    pub(crate) structure_row_count: usize,
    pub(crate) structure_block_type_counts: BTreeMap<String, usize>,
    pub(crate) structure_ocr_page_blocks: usize,
    pub(crate) structure_ocr_region_blocks: usize,
    pub(crate) structure_bbox_blocks: usize,
    pub(crate) structure_reading_order_sorted: Option<bool>,
    pub(crate) structure_order_signature: Option<String>,
    pub(crate) structure_order_first_key: Option<String>,
    pub(crate) structure_order_last_key: Option<String>,
    pub(crate) structure_baseline_dir: Option<String>,
    pub(crate) structure_parity: Option<Value>,
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
    pub(crate) document_timing_arrow_bytes: u64,
    pub(crate) document_timing_row_count: usize,
    pub(crate) document_timing_status_counts: BTreeMap<String, usize>,
    pub(crate) document_timing_phase_elapsed_ms: BTreeMap<String, f64>,
    pub(crate) document_timing_total_elapsed_ms: f64,
    pub(crate) hybrid_page_ocr_fallback_reason: Option<String>,
    pub(crate) hybrid_page_ocr_timing_report_bytes: u64,
    pub(crate) hybrid_page_ocr_timing_phase_elapsed_ms: BTreeMap<String, f64>,
    pub(crate) hybrid_page_ocr_timing_total_elapsed_ms: f64,
    pub(crate) hybrid_page_ocr_timing_ocr_shard_count: usize,
    pub(crate) hybrid_page_ocr_timing_ocr2_region_shard_count: usize,
    pub(crate) hybrid_page_ocr_timing_ocr2_region_request_count: usize,
    pub(crate) hybrid_page_ocr_timing_ocr2_region_rendered_shard_count: usize,
    pub(crate) hybrid_page_ocr_timing_ocr2_region_render_cache_hit_count: usize,
    pub(crate) hybrid_page_ocr_timing_ocr2_region_render_cache_miss_count: usize,
    pub(crate) hybrid_page_ocr_timing_ocr2_region_render_reported_elapsed_ms: f64,
    pub(crate) hybrid_page_ocr_timing_ocr2_region_pipeline_planned_render_chunk_count: usize,
    pub(crate) hybrid_page_ocr_timing_ocr2_region_pipeline_endpoint_count: usize,
    pub(crate) hybrid_page_ocr_timing_ocr2_region_pipeline_render_ahead_limit: usize,
    pub(crate) hybrid_page_ocr_timing_ocr2_region_pipeline_render_spawn_count: usize,
    pub(crate) hybrid_page_ocr_timing_ocr2_region_pipeline_render_chunk_count: usize,
    pub(crate) hybrid_page_ocr_timing_ocr2_region_pipeline_region_dispatch_count: usize,
    pub(crate) structure_authority_pages: usize,
    pub(crate) text_shortcut_pages: usize,
    pub(crate) ocr_patch_regions: usize,
    pub(crate) page_range_docling_fallback_pages: usize,
    pub(crate) page_range_docling_fallback_chunk_count: usize,
    pub(crate) page_range_docling_fallback_plan: Option<Value>,
    pub(crate) page_range_docling_fallback_chunks: Vec<Value>,
    pub(crate) page_range_docling_fallback_chunk_summary: Option<Value>,
    pub(crate) full_docling_fallback_count: usize,
    pub(crate) hybrid_page_ocr_timing_scheduler_trace: Vec<Value>,
    #[cfg(feature = "document-extract-attachment-audit")]
    pub(crate) image_attachment_audit: Option<AttachmentAudit>,
    #[cfg(feature = "document-extract-attachment-audit")]
    pub(crate) image_attachment_audit_error: Option<String>,
    #[cfg(feature = "document-extract-attachment-audit")]
    pub(crate) archive_attachment_audit: Option<ArchiveAttachmentAudit>,
    #[cfg(feature = "document-extract-attachment-audit")]
    pub(crate) archive_attachment_audit_error: Option<String>,
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
        audio_transcript_chars: 0,
        audio_transcript_timeline_marker_count: 0,
        audio_transcript_timeline_marked_rows: 0,
        structure_arrow_exists: false,
        structure_arrow_bytes: 0,
        structure_row_count: 0,
        structure_block_type_counts: BTreeMap::new(),
        structure_ocr_page_blocks: 0,
        structure_ocr_region_blocks: 0,
        structure_bbox_blocks: 0,
        structure_reading_order_sorted: None,
        structure_order_signature: None,
        structure_order_first_key: None,
        structure_order_last_key: None,
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
        document_timing_arrow_bytes: 0,
        document_timing_row_count: 0,
        document_timing_status_counts: BTreeMap::new(),
        document_timing_phase_elapsed_ms: BTreeMap::new(),
        document_timing_total_elapsed_ms: 0.0,
        hybrid_page_ocr_fallback_reason: None,
        hybrid_page_ocr_timing_report_bytes: 0,
        hybrid_page_ocr_timing_phase_elapsed_ms: BTreeMap::new(),
        hybrid_page_ocr_timing_total_elapsed_ms: 0.0,
        hybrid_page_ocr_timing_ocr_shard_count: 0,
        hybrid_page_ocr_timing_ocr2_region_shard_count: 0,
        hybrid_page_ocr_timing_ocr2_region_request_count: 0,
        hybrid_page_ocr_timing_ocr2_region_rendered_shard_count: 0,
        hybrid_page_ocr_timing_ocr2_region_render_cache_hit_count: 0,
        hybrid_page_ocr_timing_ocr2_region_render_cache_miss_count: 0,
        hybrid_page_ocr_timing_ocr2_region_render_reported_elapsed_ms: 0.0,
        hybrid_page_ocr_timing_ocr2_region_pipeline_planned_render_chunk_count: 0,
        hybrid_page_ocr_timing_ocr2_region_pipeline_endpoint_count: 0,
        hybrid_page_ocr_timing_ocr2_region_pipeline_render_ahead_limit: 0,
        hybrid_page_ocr_timing_ocr2_region_pipeline_render_spawn_count: 0,
        hybrid_page_ocr_timing_ocr2_region_pipeline_render_chunk_count: 0,
        hybrid_page_ocr_timing_ocr2_region_pipeline_region_dispatch_count: 0,
        structure_authority_pages: 0,
        text_shortcut_pages: 0,
        ocr_patch_regions: 0,
        page_range_docling_fallback_pages: 0,
        page_range_docling_fallback_chunk_count: 0,
        page_range_docling_fallback_plan: None,
        page_range_docling_fallback_chunks: Vec::new(),
        page_range_docling_fallback_chunk_summary: None,
        full_docling_fallback_count: 0,
        hybrid_page_ocr_timing_scheduler_trace: Vec::new(),
        #[cfg(feature = "document-extract-attachment-audit")]
        image_attachment_audit: None,
        #[cfg(feature = "document-extract-attachment-audit")]
        image_attachment_audit_error: None,
        #[cfg(feature = "document-extract-attachment-audit")]
        archive_attachment_audit: None,
        #[cfg(feature = "document-extract-attachment-audit")]
        archive_attachment_audit_error: None,
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
    #[cfg(feature = "document-extract-attachment-audit")]
    populate_image_attachment_audit(report);
    #[cfg(feature = "document-extract-attachment-audit")]
    populate_archive_attachment_audit(report);

    let output_dir = std::path::PathBuf::from(&report.output_dir);
    populate_hybrid_page_ocr_fallback_reason(report, output_dir.as_path())?;
    populate_hybrid_page_ocr_timing_report(report, output_dir.as_path())?;

    let resources_path = output_dir.join(DOCUMENT_RESOURCES_ARROW_CACHE_NAME);
    if let Some(batches) = read_arrow_file_batches(resources_path.as_path())? {
        report.resources_arrow_exists = true;
        report.resources_arrow_bytes = file_len(resources_path.as_path())?;
        report.resources_row_count = batches.iter().map(RecordBatch::num_rows).sum();
        report.resource_type_counts = string_counts(&batches, "resourceType")?;
        report.resource_status_counts = string_counts(&batches, "status")?;
        populate_audio_transcript_metrics(report, &batches)?;
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
        let order_audit = structure_order_audit(&batches)?;
        report.structure_reading_order_sorted = Some(order_audit.sorted);
        report.structure_order_signature = order_audit.signature;
        report.structure_order_first_key = order_audit.first_key;
        report.structure_order_last_key = order_audit.last_key;
        structure_batches = Some(batches);
    }
    #[cfg(feature = "document-extract-pdf-source-range")]
    populate_structure_parity(
        report,
        output_dir.as_path(),
        structure_batches.as_deref(),
        structure_baseline_root,
    )?;
    #[cfg(not(feature = "document-extract-pdf-source-range"))]
    {
        let _ = structure_batches.as_deref();
        populate_structure_parity(report, output_dir.as_path(), structure_baseline_root);
    }

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

    let document_timing_path = output_dir.join(DOCUMENT_TIMING_ARROW_CACHE_NAME);
    if let Some(batches) = read_arrow_file_batches(document_timing_path.as_path())? {
        report.document_timing_arrow_bytes = file_len(document_timing_path.as_path())?;
        report.document_timing_row_count = batches.iter().map(RecordBatch::num_rows).sum();
        report.document_timing_status_counts = string_counts(&batches, "status")?;
        report.document_timing_phase_elapsed_ms =
            sum_float64_by_string_column(&batches, "phase", "elapsedMs")?;
        report.document_timing_total_elapsed_ms = report
            .document_timing_phase_elapsed_ms
            .get("total")
            .copied()
            .unwrap_or_default();
    }
    Ok(())
}

fn populate_audio_transcript_metrics(
    report: &mut ArtifactReport,
    batches: &[RecordBatch],
) -> Result<(), String> {
    for batch in batches {
        let Some(resource_type_column) = batch.column_by_name("resourceType") else {
            continue;
        };
        let Some(content_column) = batch.column_by_name("content") else {
            continue;
        };
        let Some(resource_types) = resource_type_column.as_any().downcast_ref::<StringArray>()
        else {
            return Err("document resource `resourceType` column is not utf8".to_string());
        };
        let Some(contents) = content_column.as_any().downcast_ref::<StringArray>() else {
            return Err("document resource `content` column is not utf8".to_string());
        };
        for row in 0..batch.num_rows() {
            if resource_types.is_null(row)
                || contents.is_null(row)
                || resource_types.value(row) != "audio-transcript"
            {
                continue;
            }
            let content = contents.value(row);
            let marker_count = audio_transcript_timeline_marker_count(content);
            report.audio_transcript_chars = report
                .audio_transcript_chars
                .saturating_add(content.chars().count());
            report.audio_transcript_timeline_marker_count = report
                .audio_transcript_timeline_marker_count
                .saturating_add(marker_count);
            if marker_count > 0 {
                report.audio_transcript_timeline_marked_rows = report
                    .audio_transcript_timeline_marked_rows
                    .saturating_add(1);
            }
        }
    }
    Ok(())
}

fn audio_transcript_timeline_marker_count(content: &str) -> usize {
    content
        .lines()
        .filter(|line| is_audio_timeline_marker_line(line.trim_start()))
        .count()
}

fn is_audio_timeline_marker_line(line: &str) -> bool {
    let Some(marker_end) = line.find(']') else {
        return false;
    };
    let marker = &line[..=marker_end];
    marker.starts_with('[')
        && marker.contains('-')
        && marker[..marker_end]
            .split('-')
            .all(|part| part.contains(':'))
}

fn populate_hybrid_page_ocr_fallback_reason(
    report: &mut ArtifactReport,
    output_dir: &Path,
) -> Result<(), String> {
    let fallback_path = output_dir.join(HYBRID_PAGE_OCR_FALLBACK_REPORT_NAME);
    if !fallback_path.exists() {
        return Ok(());
    }
    let value = serde_json::from_slice::<Value>(
        fs::read(fallback_path.as_path())
            .map_err(|error| format!("read hybrid OCR fallback report: {error}"))?
            .as_slice(),
    )
    .map_err(|error| format!("decode hybrid OCR fallback report: {error}"))?;
    report.hybrid_page_ocr_fallback_reason = value
        .get("reason")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    Ok(())
}

fn populate_hybrid_page_ocr_timing_report(
    report: &mut ArtifactReport,
    output_dir: &Path,
) -> Result<(), String> {
    let timing_path = output_dir.join(HYBRID_PAGE_OCR_TIMING_REPORT_NAME);
    if !timing_path.exists() {
        return Ok(());
    }
    let value = serde_json::from_slice::<Value>(
        fs::read(timing_path.as_path())
            .map_err(|error| format!("read hybrid OCR timing report: {error}"))?
            .as_slice(),
    )
    .map_err(|error| format!("decode hybrid OCR timing report: {error}"))?;
    report.hybrid_page_ocr_timing_report_bytes = file_len(timing_path.as_path())?;
    report.hybrid_page_ocr_timing_total_elapsed_ms = value
        .get("totalElapsedMs")
        .and_then(Value::as_f64)
        .unwrap_or_default();
    report.hybrid_page_ocr_timing_ocr_shard_count = value
        .get("ocrShardCount")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or_default();
    report.hybrid_page_ocr_timing_ocr2_region_shard_count = value
        .get("ocr2RegionShardCount")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or_default();
    report.hybrid_page_ocr_timing_ocr2_region_request_count = value
        .get("ocr2RegionRequestCount")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or_default();
    report.hybrid_page_ocr_timing_ocr2_region_rendered_shard_count = value
        .get("ocr2RegionRenderedShardCount")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or_default();
    report.hybrid_page_ocr_timing_ocr2_region_render_cache_hit_count = value
        .get("ocr2RegionRenderCacheHitCount")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or_default();
    report.hybrid_page_ocr_timing_ocr2_region_render_cache_miss_count = value
        .get("ocr2RegionRenderCacheMissCount")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or_default();
    report.hybrid_page_ocr_timing_ocr2_region_render_reported_elapsed_ms = value
        .get("ocr2RegionRenderReportedElapsedMs")
        .and_then(Value::as_f64)
        .unwrap_or_default();
    report.hybrid_page_ocr_timing_ocr2_region_pipeline_planned_render_chunk_count = value
        .get("ocr2RegionPipelinePlannedRenderChunkCount")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or_default();
    report.hybrid_page_ocr_timing_ocr2_region_pipeline_endpoint_count = value
        .get("ocr2RegionPipelineEndpointCount")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or_default();
    report.hybrid_page_ocr_timing_ocr2_region_pipeline_render_ahead_limit = value
        .get("ocr2RegionPipelineRenderAheadLimit")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or_default();
    report.hybrid_page_ocr_timing_ocr2_region_pipeline_render_spawn_count = value
        .get("ocr2RegionPipelineRenderSpawnCount")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or_default();
    report.hybrid_page_ocr_timing_ocr2_region_pipeline_render_chunk_count = value
        .get("ocr2RegionPipelineRenderChunkCount")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or_default();
    report.hybrid_page_ocr_timing_ocr2_region_pipeline_region_dispatch_count = value
        .get("ocr2RegionPipelineRegionDispatchCount")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or_default();
    report.structure_authority_pages = timing_report_usize(&value, "structureAuthorityPages");
    report.text_shortcut_pages = timing_report_usize(&value, "textShortcutPages");
    report.ocr_patch_regions = timing_report_usize(&value, "ocrPatchRegions");
    report.page_range_docling_fallback_pages =
        timing_report_usize(&value, "pageRangeDoclingFallbackPages");
    report.page_range_docling_fallback_chunk_count =
        timing_report_usize(&value, "pageRangeDoclingFallbackChunkCount");
    report.page_range_docling_fallback_plan = value.get("pageRangeDoclingFallbackPlan").cloned();
    report.page_range_docling_fallback_chunks = value
        .get("pageRangeDoclingFallbackChunks")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    report.page_range_docling_fallback_chunk_summary =
        value.get("pageRangeDoclingFallbackChunkSummary").cloned();
    report.full_docling_fallback_count = timing_report_usize(&value, "fullDoclingFallbackCount");
    report.hybrid_page_ocr_timing_scheduler_trace = value
        .get("ocrSchedulerTrace")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if let Some(phases) = value.get("phaseElapsedMs").and_then(Value::as_object) {
        report.hybrid_page_ocr_timing_phase_elapsed_ms = phases
            .iter()
            .filter_map(|(phase, elapsed)| {
                elapsed
                    .as_f64()
                    .map(|elapsed_ms| (phase.clone(), elapsed_ms))
            })
            .collect();
    }
    Ok(())
}

fn timing_report_usize(value: &Value, key: &str) -> usize {
    value
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or_default()
}

#[cfg(feature = "document-extract-attachment-audit")]
fn populate_image_attachment_audit(report: &mut ArtifactReport) {
    let source_path = Path::new(&report.source);
    if !is_supported_image_path(source_path) {
        return;
    }
    match audit_image_attachment(source_path) {
        Ok(audit) => report.image_attachment_audit = Some(audit),
        Err(error) => report.image_attachment_audit_error = Some(error),
    }
}

#[cfg(feature = "document-extract-attachment-audit")]
fn populate_archive_attachment_audit(report: &mut ArtifactReport) {
    let source_path = Path::new(&report.source);
    if !is_supported_archive_path(source_path) {
        return;
    }
    match audit_archive_attachment(source_path) {
        Ok(audit) => report.archive_attachment_audit = Some(audit),
        Err(error) => report.archive_attachment_audit_error = Some(error),
    }
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

fn sum_float64_by_string_column(
    batches: &[RecordBatch],
    key_column_name: &str,
    value_column_name: &str,
) -> Result<BTreeMap<String, f64>, String> {
    let mut totals = BTreeMap::new();
    for batch in batches {
        let Some(key_column) = batch.column_by_name(key_column_name) else {
            continue;
        };
        let Some(value_column) = batch.column_by_name(value_column_name) else {
            continue;
        };
        let Some(keys) = key_column.as_any().downcast_ref::<StringArray>() else {
            return Err(format!(
                "document timing `{key_column_name}` column is not utf8"
            ));
        };
        let Some(values) = value_column.as_any().downcast_ref::<Float64Array>() else {
            return Err(format!(
                "document timing `{value_column_name}` column is not float64"
            ));
        };
        for row in 0..batch.num_rows() {
            if keys.is_null(row) || values.is_null(row) {
                continue;
            }
            *totals.entry(keys.value(row).to_string()).or_insert(0.0) += values.value(row);
        }
    }
    Ok(totals)
}

#[derive(Debug, Default)]
struct StructureOrderAudit {
    sorted: bool,
    signature: Option<String>,
    first_key: Option<String>,
    last_key: Option<String>,
}

fn structure_order_audit(batches: &[RecordBatch]) -> Result<StructureOrderAudit, String> {
    let mut hasher = blake3::Hasher::new();
    let mut sorted = true;
    let mut previous: Option<(i32, String, i32, String)> = None;
    let mut first_key = None;
    let mut last_key = None;
    let mut row_count = 0usize;
    for batch in batches {
        let page_index = int32_column(batch, "pageIndex")?;
        let reading_order_key = string_column(batch, "readingOrderKey")?;
        let block_index = int32_column(batch, "blockIndex")?;
        let block_id = string_column(batch, "blockId")?;
        let block_type = string_column(batch, "blockType")?;
        let resource_element_id = string_column(batch, "resourceElementId")?;
        let content = string_column(batch, "content")?;
        let status = string_column(batch, "status")?;
        for row in 0..batch.num_rows() {
            row_count += 1;
            let current = (
                int32_value(page_index, row),
                string_value(reading_order_key, row),
                int32_value(block_index, row),
                string_value(block_id, row),
            );
            if let Some(previous) = &previous
                && current < *previous
            {
                sorted = false;
            }
            let order_key = format!(
                "{:06}|{}|{:06}|{}",
                current.0, current.1, current.2, current.3
            );
            first_key.get_or_insert_with(|| order_key.clone());
            last_key = Some(order_key);
            update_structure_order_signature(
                &mut hasher,
                &[
                    current.0.to_string(),
                    current.1.clone(),
                    current.2.to_string(),
                    current.3.clone(),
                    string_value(block_type, row),
                    string_value(resource_element_id, row),
                    string_value(status, row),
                    string_value(content, row),
                ],
            );
            previous = Some(current);
        }
    }
    Ok(StructureOrderAudit {
        sorted,
        signature: (row_count > 0).then(|| hasher.finalize().to_hex().to_string()),
        first_key,
        last_key,
    })
}

fn update_structure_order_signature(hasher: &mut blake3::Hasher, fields: &[String]) {
    for field in fields {
        hasher.update(field.len().to_string().as_bytes());
        hasher.update(b":");
        hasher.update(field.as_bytes());
        hasher.update(b"\n");
    }
}

#[cfg(not(feature = "document-extract-pdf-source-range"))]
fn populate_structure_parity(
    report: &mut ArtifactReport,
    output_dir: &Path,
    baseline_root: Option<&Path>,
) {
    let Some(baseline_root) = baseline_root else {
        return;
    };
    let _ = (output_dir, baseline_root);
    report.structure_parity_error =
        Some("structure parity requires document-extract-pdf-source-range feature".to_string());
}

#[cfg(feature = "document-extract-pdf-source-range")]
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
        Ok(summary) => {
            report.structure_parity =
                Some(serde_json::to_value(summary).map_err(|error| error.to_string())?);
        }
        Err(error) => report.structure_parity_error = Some(error),
    }
    Ok(())
}

#[cfg(feature = "document-extract-pdf-source-range")]
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

#[cfg(feature = "document-extract-pdf-source-range")]
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

#[cfg(feature = "document-extract-pdf-source-range")]
fn optional_float64_value(column: &Float64Array, row: usize) -> Option<f64> {
    if column.is_null(row) {
        None
    } else {
        Some(column.value(row))
    }
}

#[cfg(feature = "document-extract-attachment-audit")]
fn write_test_png(path: &Path, width: u32, height: u32) -> Result<(), String> {
    let mut bytes = Vec::from(&b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR"[..]);
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.extend_from_slice(&[8, 2, 0, 0, 0, 0, 0, 0, 0]);
    fs::write(path, bytes).map_err(|error| error.to_string())
}

#[cfg(feature = "document-extract-attachment-audit")]
fn write_test_tar(path: &Path, files: &[(&str, &[u8])]) -> Result<(), String> {
    let mut bytes = Vec::new();
    for (member_path, content) in files {
        append_test_tar_entry(&mut bytes, member_path, content)?;
    }
    bytes.resize(bytes.len() + 1024, 0);
    fs::write(path, bytes).map_err(|error| error.to_string())
}

#[cfg(feature = "document-extract-attachment-audit")]
fn append_test_tar_entry(
    archive: &mut Vec<u8>,
    member_path: &str,
    content: &[u8],
) -> Result<(), String> {
    let mut header = [0_u8; 512];
    let path_bytes = member_path.as_bytes();
    if path_bytes.len() > 100 {
        return Err(format!("test tar member path is too long: {member_path}"));
    }
    header[..path_bytes.len()].copy_from_slice(path_bytes);
    write_tar_octal(&mut header[100..108], 0o644);
    write_tar_octal(&mut header[108..116], 0);
    write_tar_octal(&mut header[116..124], 0);
    write_tar_octal(&mut header[124..136], content.len() as u64);
    write_tar_octal(&mut header[136..148], 0);
    header[148..156].fill(b' ');
    header[156] = b'0';
    header[257..263].copy_from_slice(b"ustar\0");
    header[263..265].copy_from_slice(b"00");
    let checksum: u64 = header.iter().map(|byte| u64::from(*byte)).sum();
    let checksum_text = format!("{checksum:06o}\0 ");
    header[148..156].copy_from_slice(checksum_text.as_bytes());

    archive.extend_from_slice(&header);
    archive.extend_from_slice(content);
    let padding = (512 - (content.len() % 512)) % 512;
    archive.resize(archive.len() + padding, 0);
    Ok(())
}

#[cfg(feature = "document-extract-attachment-audit")]
fn write_tar_octal(field: &mut [u8], value: u64) {
    let width = field.len() - 1;
    let text = format!("{value:0width$o}\0");
    field.copy_from_slice(text.as_bytes());
}

#[test]
fn artifact_report_reads_document_timing_sidecar() -> Result<(), String> {
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let output_dir = temp_dir.path().join("outputs").join("image");
    fs::create_dir_all(output_dir.as_path()).map_err(|error| error.to_string())?;
    write_test_arrow_file(
        output_dir.join(DOCUMENT_TIMING_ARROW_CACHE_NAME).as_path(),
        &document_timing_test_batch()?,
    )?;

    let report = inspect_artifact_dir("fixture.png", output_dir.to_string_lossy().as_ref(), None);

    assert_eq!(report.artifact_error, None);
    assert!(report.document_timing_arrow_bytes > 0);
    assert_eq!(report.document_timing_row_count, 3);
    assert_eq!(report.document_timing_status_counts.get("ok"), Some(&3));
    assert_float_eq(
        report
            .document_timing_phase_elapsed_ms
            .get("doclingConvert")
            .copied(),
        10.0,
    )?;
    assert_float_eq(
        report
            .document_timing_phase_elapsed_ms
            .get("writeResourcesArrow")
            .copied(),
        2.0,
    )?;
    assert_float_eq(Some(report.document_timing_total_elapsed_ms), 15.0)?;
    Ok(())
}

#[test]
fn artifact_report_reads_hybrid_page_ocr_timing_sidecar() -> Result<(), String> {
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let output_dir = temp_dir.path().join("outputs").join("pdf");
    fs::create_dir_all(output_dir.as_path()).map_err(|error| error.to_string())?;
    write_hybrid_page_ocr_timing_sidecar(output_dir.as_path())?;

    let report = inspect_artifact_dir("fixture.pdf", output_dir.to_string_lossy().as_ref(), None);

    assert_hybrid_page_ocr_timing_report(&report)
}

fn write_hybrid_page_ocr_timing_sidecar(output_dir: &Path) -> Result<(), String> {
    fs::write(
        output_dir
            .join(HYBRID_PAGE_OCR_TIMING_REPORT_NAME)
            .as_path(),
        serde_json::to_vec(&serde_json::json!({
            "schema": "xiuxian_wendao.hybrid_page_ocr_timing.v1",
            "ocrShardCount": 27,
            "ocr2RegionShardCount": 6,
            "ocr2RegionRequestCount": 6,
            "ocr2RegionRenderedShardCount": 6,
            "ocr2RegionRenderCacheHitCount": 6,
            "ocr2RegionRenderCacheMissCount": 0,
            "ocr2RegionRenderReportedElapsedMs": 125.5,
            "ocr2RegionPipelinePlannedRenderChunkCount": 3,
            "ocr2RegionPipelineEndpointCount": 4,
            "ocr2RegionPipelineRenderAheadLimit": 3,
            "ocr2RegionPipelineRenderSpawnCount": 3,
            "ocr2RegionPipelineRenderChunkCount": 3,
            "ocr2RegionPipelineRegionDispatchCount": 2,
            "structureAuthorityPages": 4,
            "textShortcutPages": 2,
            "ocrPatchRegions": 3,
            "pageRangeDoclingFallbackPages": 5,
            "pageRangeDoclingFallbackChunkCount": 2,
            "pageRangeDoclingFallbackPlan": {
                "strategy": "source-profile-weighted",
                "targetChunkCount": 4,
                "fallbackPageCount": 5,
                "rangeCount": 2,
                "chunkSize": null,
                "sourceProfileUsed": true,
                "ranges": [
                    {
                        "pageStart": 0,
                        "pageEnd": 2,
                        "oneBasedStart": 1,
                        "oneBasedEnd": 3,
                    },
                    {
                        "pageStart": 3,
                        "pageEnd": 4,
                        "oneBasedStart": 4,
                        "oneBasedEnd": 5,
                    },
                ],
            },
            "pageRangeDoclingFallbackChunks": [
                {
                    "pageStart": 0,
                    "pageEnd": 2,
                    "oneBasedStart": 1,
                    "oneBasedEnd": 3,
                    "elapsedMs": 1200.0,
                    "resourceRows": 9,
                },
                {
                    "pageStart": 3,
                    "pageEnd": 4,
                    "oneBasedStart": 4,
                    "oneBasedEnd": 5,
                    "elapsedMs": 4400.0,
                    "resourceRows": 12,
                },
            ],
            "pageRangeDoclingFallbackChunkSummary": {
                "chunkCount": 2,
                "elapsedMsMax": 4400.0,
                "elapsedMsTotal": 5600.0,
                "resourceRows": 21,
                "longestPageStart": 3,
                "longestPageEnd": 4,
                "longestOneBasedStart": 4,
                "longestOneBasedEnd": 5,
                "longestResourceRows": 12,
            },
            "fullDoclingFallbackCount": 0,
            "ocrSchedulerTrace": [
                {
                    "lane": "source-pdf-page-range",
                    "shardCount": 7,
                    "pageStart": 0,
                    "pageEnd": 6,
                    "latencyMs": 19327.0,
                    "textCharCount": 48879,
                },
            ],
            "totalElapsedMs": 5600.0,
            "phaseElapsedMs": {
                "regionMaterialize": 5550.0,
                "ocrScheduler": 3.5,
                "total": 5600.0,
            },
        }))
        .map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn assert_hybrid_page_ocr_timing_report(report: &ArtifactReport) -> Result<(), String> {
    assert_eq!(report.artifact_error, None);
    assert!(report.hybrid_page_ocr_timing_report_bytes > 0);
    assert_eq!(report.hybrid_page_ocr_timing_ocr_shard_count, 27);
    assert_eq!(report.hybrid_page_ocr_timing_ocr2_region_shard_count, 6);
    assert_eq!(report.hybrid_page_ocr_timing_ocr2_region_request_count, 6);
    assert_eq!(
        report.hybrid_page_ocr_timing_ocr2_region_rendered_shard_count,
        6
    );
    assert_eq!(
        report.hybrid_page_ocr_timing_ocr2_region_render_cache_hit_count,
        6
    );
    assert_eq!(
        report.hybrid_page_ocr_timing_ocr2_region_render_cache_miss_count,
        0
    );
    assert_float_eq(
        Some(report.hybrid_page_ocr_timing_ocr2_region_render_reported_elapsed_ms),
        125.5,
    )?;
    assert_eq!(
        report.hybrid_page_ocr_timing_ocr2_region_pipeline_planned_render_chunk_count,
        3
    );
    assert_eq!(
        report.hybrid_page_ocr_timing_ocr2_region_pipeline_endpoint_count,
        4
    );
    assert_eq!(
        report.hybrid_page_ocr_timing_ocr2_region_pipeline_render_ahead_limit,
        3
    );
    assert_eq!(
        report.hybrid_page_ocr_timing_ocr2_region_pipeline_render_spawn_count,
        3
    );
    assert_eq!(
        report.hybrid_page_ocr_timing_ocr2_region_pipeline_render_chunk_count,
        3
    );
    assert_eq!(
        report.hybrid_page_ocr_timing_ocr2_region_pipeline_region_dispatch_count,
        2
    );
    assert_eq!(report.structure_authority_pages, 4);
    assert_eq!(report.text_shortcut_pages, 2);
    assert_eq!(report.ocr_patch_regions, 3);
    assert_eq!(report.page_range_docling_fallback_pages, 5);
    assert_eq!(report.page_range_docling_fallback_chunk_count, 2);
    assert_eq!(
        report
            .page_range_docling_fallback_plan
            .as_ref()
            .and_then(|plan| plan.get("strategy"))
            .and_then(Value::as_str),
        Some("source-profile-weighted")
    );
    assert_eq!(report.page_range_docling_fallback_chunks.len(), 2);
    assert_eq!(
        report.page_range_docling_fallback_chunks[1]["resourceRows"],
        12
    );
    assert_eq!(
        report
            .page_range_docling_fallback_chunk_summary
            .as_ref()
            .and_then(|summary| summary.get("longestPageStart"))
            .and_then(Value::as_u64),
        Some(3)
    );
    assert_eq!(report.full_docling_fallback_count, 0);
    assert_eq!(report.hybrid_page_ocr_timing_scheduler_trace.len(), 1);
    assert_eq!(
        report.hybrid_page_ocr_timing_scheduler_trace[0]["lane"],
        "source-pdf-page-range"
    );
    assert_eq!(
        report.hybrid_page_ocr_timing_scheduler_trace[0]["textCharCount"],
        48879
    );
    assert_float_eq(Some(report.hybrid_page_ocr_timing_total_elapsed_ms), 5600.0)?;
    assert_float_eq(
        report
            .hybrid_page_ocr_timing_phase_elapsed_ms
            .get("regionMaterialize")
            .copied(),
        5550.0,
    )?;
    assert_float_eq(
        report
            .hybrid_page_ocr_timing_phase_elapsed_ms
            .get("ocrScheduler")
            .copied(),
        3.5,
    )?;
    Ok(())
}

fn assert_float_eq(actual: Option<f64>, expected: f64) -> Result<(), String> {
    let actual = actual.ok_or_else(|| format!("missing expected float value {expected}"))?;
    if (actual - expected).abs() > f64::EPSILON {
        return Err(format!("expected {expected}, got {actual}"));
    }
    Ok(())
}

#[test]
#[cfg(feature = "document-extract-attachment-audit")]
fn artifact_report_reads_image_attachment_audit() -> Result<(), String> {
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let output_dir = temp_dir.path().join("outputs").join("image");
    fs::create_dir_all(output_dir.as_path()).map_err(|error| error.to_string())?;
    let image_path = temp_dir.path().join("source.png");
    write_test_png(image_path.as_path(), 320, 240)?;

    let report = inspect_artifact_dir(
        image_path.to_string_lossy().as_ref(),
        output_dir.to_string_lossy().as_ref(),
        None,
    );

    assert_eq!(report.artifact_error, None);
    assert_eq!(report.image_attachment_audit_error, None);
    let audit = report
        .image_attachment_audit
        .ok_or_else(|| "expected image attachment audit".to_string())?;
    assert_eq!(audit.format, "png");
    assert_eq!(audit.width_px, Some(320));
    assert_eq!(audit.height_px, Some(240));
    assert_eq!(audit.pixel_count, Some(76_800));
    assert_eq!(
        audit.rust_acceleration_candidate,
        "image_ocr_cache_candidate"
    );
    Ok(())
}

#[test]
#[cfg(feature = "document-extract-attachment-audit")]
fn artifact_report_reads_archive_attachment_audit() -> Result<(), String> {
    let temp_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let output_dir = temp_dir.path().join("outputs").join("archive");
    fs::create_dir_all(output_dir.as_path()).map_err(|error| error.to_string())?;
    let archive_path = temp_dir.path().join("source.tar");
    write_test_tar(
        archive_path.as_path(),
        &[
            ("book/mets.xml", b"<mets/>".as_slice()),
            ("book/images/page0001.tif", b"tiff-image".as_slice()),
            ("book/text/page0001.txt", b"text".as_slice()),
        ],
    )?;

    let report = inspect_artifact_dir(
        archive_path.to_string_lossy().as_ref(),
        output_dir.to_string_lossy().as_ref(),
        None,
    );

    assert_eq!(report.artifact_error, None);
    assert_eq!(report.archive_attachment_audit_error, None);
    let audit = report
        .archive_attachment_audit
        .ok_or_else(|| "expected archive attachment audit".to_string())?;
    assert_eq!(audit.archive_format, "tar");
    assert_eq!(audit.member_count, 3);
    assert_eq!(audit.xml_member_count, 1);
    assert_eq!(audit.image_member_count, 1);
    assert_eq!(
        audit.likely_mets_member_path.as_deref(),
        Some("book/mets.xml")
    );
    assert_eq!(
        audit.rust_acceleration_candidate,
        "mets_gbs_member_manifest_candidate"
    );
    Ok(())
}

#[test]
#[cfg(feature = "document-extract-pdf-source-range")]
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
    assert!(report.structure_order_signature.is_some());
    assert_eq!(
        report.structure_order_first_key.as_deref(),
        Some("000000|000000.000000|000000|native")
    );
    assert_eq!(
        report.structure_order_last_key.as_deref(),
        Some("000000|000000.000001|000001|region")
    );
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
    assert_eq!(
        parity.get("baselineBlockCount").and_then(Value::as_u64),
        Some(1)
    );
    assert_eq!(
        parity.get("candidateBlockCount").and_then(Value::as_u64),
        Some(2)
    );
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

fn document_timing_test_batch() -> Result<RecordBatch, String> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("contractVersion", DataType::Utf8, true),
        Field::new("sourcePath", DataType::Utf8, true),
        Field::new("sourceSuffix", DataType::Utf8, true),
        Field::new("phase", DataType::Utf8, true),
        Field::new("elapsedMs", DataType::Float64, true),
        Field::new("status", DataType::Utf8, true),
        Field::new("detail", DataType::Utf8, true),
        Field::new("resourceRows", DataType::Int32, true),
        Field::new("structureRows", DataType::Int32, true),
    ]));
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(vec![
                "xiuxian_wendao.document_timing.v1",
                "xiuxian_wendao.document_timing.v1",
                "xiuxian_wendao.document_timing.v1",
            ])) as ArrayRef,
            Arc::new(StringArray::from(vec![
                "fixture.png",
                "fixture.png",
                "fixture.png",
            ])) as ArrayRef,
            Arc::new(StringArray::from(vec![".png", ".png", ".png"])) as ArrayRef,
            Arc::new(StringArray::from(vec![
                "doclingConvert",
                "writeResourcesArrow",
                "total",
            ])) as ArrayRef,
            Arc::new(Float64Array::from(vec![Some(10.0), Some(2.0), Some(15.0)])) as ArrayRef,
            Arc::new(StringArray::from(vec!["ok", "ok", "ok"])) as ArrayRef,
            Arc::new(StringArray::from(vec!["", "", ""])) as ArrayRef,
            Arc::new(Int32Array::from(vec![0, 0, 1])) as ArrayRef,
            Arc::new(Int32Array::from(vec![0, 0, 1])) as ArrayRef,
        ],
    )
    .map_err(|error| error.to_string())
}

#[cfg(feature = "document-extract-pdf-source-range")]
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

#[cfg(feature = "document-extract-pdf-source-range")]
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

#[cfg(feature = "document-extract-pdf-source-range")]
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

#[cfg(feature = "document-extract-pdf-source-range")]
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
