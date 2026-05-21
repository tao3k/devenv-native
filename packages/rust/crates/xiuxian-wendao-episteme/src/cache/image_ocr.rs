//! Image OCR cache materialization for Episteme source contracts.

use std::{collections::BTreeMap, fs, io::Read, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    materialization_support::{
        display_path, increment, non_empty_or, normalized_text, sha256_text, write_json,
    },
    path::{resolve_existing_corpus_file, resolve_run_output_path},
    task::{EpistemeCacheTask, read_tasks_tsv, task_extension},
};

/// Source-contract extraction route for image OCR evidence.
pub const EPISTEME_IMAGE_OCR_ROUTE: &str = "image_ocr_evidence";
/// Default JSONL filename emitted by the analyzer image OCR adapter.
pub const EPISTEME_IMAGE_OCR_RESULTS_JSONL: &str = "ocr_results.jsonl";
/// Wrapper report schema for the Studio CLI orchestration layer.
pub const EPISTEME_IMAGE_OCR_WRAPPER_SCHEMA: &str =
    "xiuxian_wendao.episteme_image_ocr_cache_execution.v1";

const CACHE_OUTPUT_SCHEMA: &str = "xiuxian_wendao.episteme_evidence_text_cache.v1";
const CACHE_RECEIPT_SCHEMA: &str = "xiuxian_wendao.episteme_image_ocr_cache_receipt.v1";
const OUTPUT_CONTRACT: &str = "cache_only_no_rdf_promotion";
const SUPPORTED_EXTENSIONS: [&str; 3] = ["jpeg", "jpg", "png"];

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(transparent)]
struct JsonBool(bool);

impl From<bool> for JsonBool {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

/// Summary for an image OCR cache materialization pass.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeImageOcrCacheBridgeReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Whether cache materialization was skipped by a dry-run caller.
    pub skipped: bool,
    /// Whether all planned rows succeeded.
    pub passed: bool,
    /// Analyzer JSONL path.
    pub ocr_results_jsonl: String,
    /// Output directory path.
    pub outputs_dir: String,
    /// Receipt path.
    pub receipt_path: String,
    /// Number of planned tasks.
    pub attempted_count: usize,
    /// Number of successful cache rows.
    pub succeeded_count: usize,
    /// Number of failed cache rows.
    pub failed_count: usize,
    /// Count by output status.
    pub status_counts: BTreeMap<String, usize>,
    /// Count by extractor name.
    pub extractor_counts: BTreeMap<String, usize>,
    /// Count by source extension.
    pub extension_counts: BTreeMap<String, usize>,
    /// Total extracted text characters across successful rows.
    pub total_text_chars: usize,
    /// Cache rows are evidence only and cannot be promoted directly to RDF.
    pub raw_to_rdf_promotion_allowed: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
struct ImageOcrJsonlRow {
    queue_id: String,
    text: String,
    ocr_engine: Option<String>,
    ocr_profile: Option<String>,
    text_mime_type: Option<String>,
    source_sha256: Option<String>,
}

#[derive(Debug, Serialize)]
struct ImageOcrCacheOutput {
    schema_version: &'static str,
    status: &'static str,
    queue_id: String,
    file_id: String,
    relative_path: String,
    extension: String,
    category: String,
    language: String,
    extraction_route: String,
    route_family: &'static str,
    support_state: &'static str,
    source_sha256: String,
    source_hash_matched: JsonBool,
    extractor: String,
    ocr_engine: String,
    ocr_profile: String,
    text_mime_type: String,
    output_contract: &'static str,
    ocr_required: JsonBool,
    ocr_executed: JsonBool,
    raw_content_extracted: JsonBool,
    raw_to_rdf_promotion_allowed: JsonBool,
    ontology_truth: JsonBool,
    review_status: &'static str,
    promotion_status: &'static str,
    text_char_count: usize,
    text_sha256: String,
    extracted_text: String,
    image_format: String,
    image_width: u32,
    image_height: u32,
}

#[derive(Debug, Serialize)]
struct ImageOcrCacheFailureOutput {
    schema_version: &'static str,
    status: &'static str,
    queue_id: String,
    file_id: String,
    relative_path: String,
    extension: String,
    category: String,
    language: String,
    extraction_route: String,
    route_family: &'static str,
    support_state: &'static str,
    source_sha256: String,
    source_hash_matched: JsonBool,
    output_contract: &'static str,
    ocr_required: JsonBool,
    ocr_executed: JsonBool,
    raw_content_extracted: JsonBool,
    raw_to_rdf_promotion_allowed: JsonBool,
    ontology_truth: JsonBool,
    text_char_count: usize,
    error: String,
}

#[derive(Debug, Serialize)]
struct ImageOcrCacheReceipt {
    schema_version: &'static str,
    extraction_executed: bool,
    raw_content_extracted: bool,
    image_ocr_cache_execution: EpistemeImageOcrCacheBridgeReport,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ImageDimensions {
    format: &'static str,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
struct ImageOcrCacheWriteSummary {
    status_counts: BTreeMap<String, usize>,
    extractor_counts: BTreeMap<String, usize>,
    extension_counts: BTreeMap<String, usize>,
    total_text_chars: usize,
    succeeded_count: usize,
    failed_count: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum ImageOcrTaskWriteOutcome {
    Succeeded {
        extractor: String,
        text_char_count: usize,
    },
    Failed,
}

/// Build a dry-run bridge report without writing cache rows.
#[must_use]
pub fn skipped_image_ocr_cache_bridge_report(
    ocr_results_jsonl: &Path,
    outputs_dir: &Path,
    receipt_path: &Path,
) -> EpistemeImageOcrCacheBridgeReport {
    EpistemeImageOcrCacheBridgeReport {
        schema_version: CACHE_RECEIPT_SCHEMA,
        skipped: true,
        passed: true,
        ocr_results_jsonl: display_path(ocr_results_jsonl),
        outputs_dir: display_path(outputs_dir),
        receipt_path: display_path(receipt_path),
        attempted_count: 0,
        succeeded_count: 0,
        failed_count: 0,
        status_counts: BTreeMap::new(),
        extractor_counts: BTreeMap::new(),
        extension_counts: BTreeMap::new(),
        total_text_chars: 0,
        raw_to_rdf_promotion_allowed: false,
    }
}

/// Validate that selected tasks are supported by the image OCR route.
///
/// # Errors
///
/// Returns an error when any task extension is outside the supported
/// `jpg/jpeg/png` set.
pub fn validate_image_ocr_tasks(tasks: &[EpistemeCacheTask]) -> Result<()> {
    let unsupported = tasks
        .iter()
        .filter_map(|task| {
            let extension = task_extension(task);
            (!SUPPORTED_EXTENSIONS.contains(&extension.as_str()))
                .then(|| format!("{} ({extension})", task.queue_id))
        })
        .collect::<Vec<_>>();
    if unsupported.is_empty() {
        return Ok(());
    }
    anyhow::bail!(
        "image OCR cache selected unsupported image tasks: {}. Use a selection run/category that contains only jpg/jpeg/png tasks.",
        unsupported.join(", ")
    )
}

/// Read image OCR cache tasks from the stable extraction `tasks.tsv`.
///
/// # Errors
///
/// Returns an error when the TSV is missing, has an unexpected header, or has
/// malformed task rows.
pub fn read_image_ocr_tasks_tsv(path: &Path) -> Result<Vec<EpistemeCacheTask>> {
    read_tasks_tsv(path, "image OCR")
}

/// Write deterministic cache rows from analyzer image OCR JSONL output.
///
/// # Errors
///
/// Returns an error when the analyzer JSONL is malformed, contains unknown task
/// ids, or a planned output path escapes the run outputs directory.
pub fn write_image_ocr_cache_outputs(
    tasks: &[EpistemeCacheTask],
    ocr_results_jsonl: &Path,
    run_dir: &Path,
    corpus_root: &Path,
) -> Result<EpistemeImageOcrCacheBridgeReport> {
    let outputs_dir = run_dir.join("outputs");
    fs::create_dir_all(&outputs_dir)
        .with_context(|| format!("failed to create `{}`", outputs_dir.display()))?;
    let receipt_path = run_dir.join("image_ocr_cache_receipt.json");
    let rows = read_image_ocr_jsonl(ocr_results_jsonl)?;
    let tasks_by_queue_id = tasks
        .iter()
        .map(|task| (task.queue_id.as_str(), task))
        .collect::<BTreeMap<_, _>>();
    let unknown = rows
        .keys()
        .filter(|queue_id| !tasks_by_queue_id.contains_key(queue_id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        anyhow::bail!(
            "image OCR results reference unknown queue ids: {}",
            unknown.join(", ")
        );
    }

    let summary = write_image_ocr_cache_task_outputs(tasks, &rows, run_dir, corpus_root)?;

    let report = EpistemeImageOcrCacheBridgeReport {
        schema_version: CACHE_RECEIPT_SCHEMA,
        skipped: false,
        passed: summary.failed_count == 0,
        ocr_results_jsonl: display_path(ocr_results_jsonl),
        outputs_dir: display_path(&outputs_dir),
        receipt_path: display_path(&receipt_path),
        attempted_count: tasks.len(),
        succeeded_count: summary.succeeded_count,
        failed_count: summary.failed_count,
        status_counts: summary.status_counts,
        extractor_counts: summary.extractor_counts,
        extension_counts: summary.extension_counts,
        total_text_chars: summary.total_text_chars,
        raw_to_rdf_promotion_allowed: false,
    };
    let receipt = ImageOcrCacheReceipt {
        schema_version: CACHE_RECEIPT_SCHEMA,
        extraction_executed: true,
        raw_content_extracted: report.succeeded_count > 0,
        image_ocr_cache_execution: report.clone(),
    };
    write_json(&receipt_path, &receipt)?;
    Ok(report)
}

fn write_image_ocr_cache_task_outputs(
    tasks: &[EpistemeCacheTask],
    rows: &BTreeMap<String, ImageOcrJsonlRow>,
    run_dir: &Path,
    corpus_root: &Path,
) -> Result<ImageOcrCacheWriteSummary> {
    tasks
        .iter()
        .map(|task| write_and_summarize_image_ocr_task(task, rows, run_dir, corpus_root))
        .try_fold(ImageOcrCacheWriteSummary::default(), |summary, next| {
            Ok(merge_image_ocr_summary(summary, next?))
        })
}

fn write_and_summarize_image_ocr_task(
    task: &EpistemeCacheTask,
    rows: &BTreeMap<String, ImageOcrJsonlRow>,
    run_dir: &Path,
    corpus_root: &Path,
) -> Result<ImageOcrCacheWriteSummary> {
    let extension = task_extension(task);
    let outcome = write_one_image_ocr_cache_output(
        task,
        &extension,
        rows.get(&task.queue_id),
        run_dir,
        corpus_root,
    )?;
    let mut summary = ImageOcrCacheWriteSummary::default();
    increment(&mut summary.extension_counts, &extension);
    match outcome {
        ImageOcrTaskWriteOutcome::Succeeded {
            extractor,
            text_char_count,
        } => {
            increment(&mut summary.status_counts, "succeeded");
            increment(&mut summary.extractor_counts, &extractor);
            summary.total_text_chars = text_char_count;
            summary.succeeded_count = 1;
        }
        ImageOcrTaskWriteOutcome::Failed => {
            increment(&mut summary.status_counts, "failed");
            summary.failed_count = 1;
        }
    }
    Ok(summary)
}

fn merge_image_ocr_summary(
    mut left: ImageOcrCacheWriteSummary,
    right: ImageOcrCacheWriteSummary,
) -> ImageOcrCacheWriteSummary {
    for (status, count) in right.status_counts {
        *left.status_counts.entry(status).or_insert(0) += count;
    }
    for (extractor, count) in right.extractor_counts {
        *left.extractor_counts.entry(extractor).or_insert(0) += count;
    }
    for (extension, count) in right.extension_counts {
        *left.extension_counts.entry(extension).or_insert(0) += count;
    }
    left.total_text_chars += right.total_text_chars;
    left.succeeded_count += right.succeeded_count;
    left.failed_count += right.failed_count;
    left
}

fn write_one_image_ocr_cache_output(
    task: &EpistemeCacheTask,
    extension: &str,
    row: Option<&ImageOcrJsonlRow>,
    run_dir: &Path,
    corpus_root: &Path,
) -> Result<ImageOcrTaskWriteOutcome> {
    let output_path = resolve_run_output_path(run_dir, &task.planned_output_path, &task.queue_id)?;
    let (source_hash, dimensions) = match build_task_context(task, extension, corpus_root) {
        Ok(value) => value,
        Err(error) => {
            write_failure_output(&output_path, task, extension, false, &error)?;
            return Ok(ImageOcrTaskWriteOutcome::Failed);
        }
    };
    let Some(row) = row else {
        write_failure_output(
            &output_path,
            task,
            extension,
            true,
            "image OCR result row is missing",
        )?;
        return Ok(ImageOcrTaskWriteOutcome::Failed);
    };
    if let Err(error) = validate_result_row(task, row, &source_hash) {
        write_failure_output(&output_path, task, extension, false, &error)?;
        return Ok(ImageOcrTaskWriteOutcome::Failed);
    }
    let text = normalized_text(&row.text);
    if text.is_empty() {
        write_failure_output(
            &output_path,
            task,
            extension,
            true,
            "image OCR result text is empty",
        )?;
        return Ok(ImageOcrTaskWriteOutcome::Failed);
    }

    let ocr_engine = non_empty_or(row.ocr_engine.as_deref(), "external-ocr");
    let text_char_count = text.chars().count();
    let output = ImageOcrCacheOutput {
        schema_version: CACHE_OUTPUT_SCHEMA,
        status: "succeeded",
        queue_id: task.queue_id.clone(),
        file_id: task.file_id.clone(),
        relative_path: task.relative_path.clone(),
        extension: extension.to_string(),
        category: task.category.as_str().to_string(),
        language: task.language.clone(),
        extraction_route: task.extraction_route.clone(),
        route_family: "image",
        support_state: "planned",
        source_sha256: source_hash,
        source_hash_matched: true.into(),
        extractor: ocr_engine.clone(),
        ocr_engine: ocr_engine.clone(),
        ocr_profile: non_empty_or(row.ocr_profile.as_deref(), "image-ocr-cache-v1"),
        text_mime_type: non_empty_or(row.text_mime_type.as_deref(), "text/markdown"),
        output_contract: OUTPUT_CONTRACT,
        ocr_required: false.into(),
        ocr_executed: true.into(),
        raw_content_extracted: true.into(),
        raw_to_rdf_promotion_allowed: false.into(),
        ontology_truth: false.into(),
        review_status: "review_required",
        promotion_status: "blocked_pending_review",
        text_char_count,
        text_sha256: sha256_text(&text),
        extracted_text: text,
        image_format: dimensions.format.to_string(),
        image_width: dimensions.width,
        image_height: dimensions.height,
    };
    write_json(&output_path, &output)?;
    Ok(ImageOcrTaskWriteOutcome::Succeeded {
        extractor: ocr_engine,
        text_char_count,
    })
}

fn build_task_context(
    task: &EpistemeCacheTask,
    extension: &str,
    corpus_root: &Path,
) -> Result<(String, ImageDimensions), String> {
    if task.extraction_route != EPISTEME_IMAGE_OCR_ROUTE {
        return Err(format!(
            "image OCR result targeted non-image route `{}`",
            task.extraction_route
        ));
    }
    if task.output_contract != OUTPUT_CONTRACT {
        return Err(format!(
            "image OCR task has unsupported output contract `{}`",
            task.output_contract
        ));
    }
    let source_path =
        resolve_existing_corpus_file(corpus_root, &task.relative_path, &task.queue_id)
            .map_err(|error| error.to_string())?;
    if !source_path.is_file() {
        return Err("source image is missing".to_string());
    }
    let source_hash = sha256_file(&source_path).map_err(|error| error.to_string())?;
    if source_hash != task.source_sha256 {
        return Err("source sha256 drift".to_string());
    }
    let dimensions =
        image_dimensions(&source_path, extension).map_err(|error| error.to_string())?;
    Ok((source_hash, dimensions))
}

fn read_image_ocr_jsonl(path: &Path) -> Result<BTreeMap<String, ImageOcrJsonlRow>> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let mut rows = BTreeMap::new();
    for (index, line) in raw.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let row = serde_json::from_str::<ImageOcrJsonlRow>(line).with_context(|| {
            format!(
                "failed to parse image OCR JSONL line {} in `{}`",
                index + 1,
                path.display()
            )
        })?;
        let queue_id = row.queue_id.trim().to_string();
        if queue_id.is_empty() {
            anyhow::bail!("image OCR JSONL line {} missing queue_id", index + 1);
        }
        if rows.insert(queue_id.clone(), row).is_some() {
            anyhow::bail!("duplicate image OCR queue_id `{queue_id}`");
        }
    }
    Ok(rows)
}

fn validate_result_row(
    task: &EpistemeCacheTask,
    row: &ImageOcrJsonlRow,
    source_hash: &str,
) -> Result<(), String> {
    if let Some(row_hash) = row
        .source_sha256
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        && row_hash != task.source_sha256
    {
        return Err(format!(
            "image OCR source sha256 drift for `{}`",
            task.queue_id
        ));
    }
    if source_hash != task.source_sha256 {
        return Err(format!("source sha256 drift for `{}`", task.queue_id));
    }
    Ok(())
}

fn write_failure_output(
    path: &Path,
    task: &EpistemeCacheTask,
    extension: &str,
    source_hash_matched: bool,
    error: &str,
) -> Result<()> {
    let output = ImageOcrCacheFailureOutput {
        schema_version: CACHE_OUTPUT_SCHEMA,
        status: "failed",
        queue_id: task.queue_id.clone(),
        file_id: task.file_id.clone(),
        relative_path: task.relative_path.clone(),
        extension: extension.to_string(),
        category: task.category.as_str().to_string(),
        language: task.language.clone(),
        extraction_route: task.extraction_route.clone(),
        route_family: "image",
        support_state: "planned",
        source_sha256: task.source_sha256.clone(),
        source_hash_matched: source_hash_matched.into(),
        output_contract: OUTPUT_CONTRACT,
        ocr_required: true.into(),
        ocr_executed: false.into(),
        raw_content_extracted: false.into(),
        raw_to_rdf_promotion_allowed: false.into(),
        ontology_truth: false.into(),
        text_char_count: 0,
        error: error.to_string(),
    };
    write_json(path, &output)
}

fn image_dimensions(path: &Path, extension: &str) -> Result<ImageDimensions> {
    match extension {
        "jpg" | "jpeg" => jpeg_dimensions(path),
        "png" => png_dimensions(path),
        _ => anyhow::bail!("unsupported image OCR extension: {extension}"),
    }
}

fn jpeg_dimensions(path: &Path) -> Result<ImageDimensions> {
    let mut file =
        fs::File::open(path).with_context(|| format!("failed to open `{}`", path.display()))?;
    let mut marker = [0u8; 2];
    file.read_exact(&mut marker)?;
    if marker != [0xff, 0xd8] {
        anyhow::bail!("invalid JPEG signature");
    }
    loop {
        let mut prefix = [0u8; 1];
        if file.read_exact(&mut prefix).is_err() {
            break;
        }
        while prefix[0] != 0xff {
            if file.read_exact(&mut prefix).is_err() {
                anyhow::bail!("JPEG dimension marker not found");
            }
        }
        let mut marker_byte = [0u8; 1];
        file.read_exact(&mut marker_byte)?;
        while marker_byte[0] == 0xff {
            file.read_exact(&mut marker_byte)?;
        }
        let marker = marker_byte[0];
        if marker == 0x00 {
            continue;
        }
        if marker == 0xd9 {
            break;
        }
        if marker == 0x01 || (0xd0..=0xd8).contains(&marker) {
            continue;
        }
        let mut length_bytes = [0u8; 2];
        file.read_exact(&mut length_bytes)?;
        let segment_length = u16::from_be_bytes(length_bytes);
        if segment_length < 2 {
            anyhow::bail!("invalid JPEG segment length");
        }
        let payload_length = usize::from(segment_length - 2);
        if is_jpeg_sof_marker(marker) {
            let mut payload = vec![0u8; payload_length];
            file.read_exact(&mut payload)?;
            if payload.len() < 5 {
                anyhow::bail!("invalid JPEG SOF payload");
            }
            let height = u32::from(u16::from_be_bytes([payload[1], payload[2]]));
            let width = u32::from(u16::from_be_bytes([payload[3], payload[4]]));
            if width == 0 || height == 0 {
                anyhow::bail!("invalid JPEG dimensions");
            }
            return Ok(ImageDimensions {
                format: "jpeg",
                width,
                height,
            });
        }
        let mut discard = vec![0u8; payload_length];
        file.read_exact(&mut discard)?;
    }
    anyhow::bail!("JPEG dimension marker not found")
}

fn png_dimensions(path: &Path) -> Result<ImageDimensions> {
    let data = fs::read(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    if data.len() < 24 || &data[..8] != b"\x89PNG\r\n\x1a\n" {
        anyhow::bail!("invalid PNG signature");
    }
    if &data[12..16] != b"IHDR" {
        anyhow::bail!("PNG missing IHDR header");
    }
    let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
    if width == 0 || height == 0 {
        anyhow::bail!("invalid PNG dimensions");
    }
    Ok(ImageDimensions {
        format: "png",
        width,
        height,
    })
}

fn is_jpeg_sof_marker(marker: u8) -> bool {
    matches!(
        marker,
        0xc0 | 0xc1 | 0xc2 | 0xc3 | 0xc5 | 0xc6 | 0xc7 | 0xc9 | 0xca | 0xcb | 0xcd | 0xce | 0xcf
    )
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file =
        fs::File::open(path).with_context(|| format!("failed to open `{}`", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1024 * 1024].into_boxed_slice();
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
