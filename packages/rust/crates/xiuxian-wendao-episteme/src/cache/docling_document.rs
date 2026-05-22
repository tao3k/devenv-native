//! Docling document cache materialization for Episteme source contracts.

use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::{
    materialization_support::{
        display_path, increment, non_empty_or, normalized_text, sha256_file, sha256_text,
        write_json,
    },
    path::{resolve_existing_corpus_file, resolve_run_output_path},
    task::{EpistemeCacheTask, read_tasks_tsv, task_extension},
};

/// Source-contract extraction route for Docling-compatible document evidence.
pub const EPISTEME_DOCLING_DOCUMENT_ROUTE: &str = "document_text_evidence";
/// Default JSONL filename emitted by the analyzer Docling adapter.
pub const EPISTEME_DOCLING_DOCUMENT_RESULTS_JSONL: &str = "document_results.jsonl";
/// Wrapper report schema for the Studio CLI orchestration layer.
pub const EPISTEME_DOCLING_DOCUMENT_WRAPPER_SCHEMA: &str =
    "xiuxian_wendao.episteme_docling_document_cache_execution.v1";

const CACHE_OUTPUT_SCHEMA: &str = "xiuxian_wendao.episteme_evidence_text_cache.v1";
const CACHE_RECEIPT_SCHEMA: &str = "xiuxian_wendao.episteme_docling_document_cache_receipt.v1";
const OUTPUT_CONTRACT: &str = "cache_only_no_rdf_promotion";
const SUPPORTED_EXTENSIONS: [&str; 4] = ["docx", "pdf", "pptx", "xlsx"];

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(transparent)]
struct JsonBool(bool);

impl From<bool> for JsonBool {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

/// Summary for a Docling document cache materialization pass.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeDoclingDocumentCacheBridgeReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Whether cache materialization was skipped by a dry-run caller.
    pub skipped: bool,
    /// Whether all planned rows succeeded.
    pub passed: bool,
    /// Analyzer JSONL path.
    pub document_results_jsonl: String,
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
struct DoclingDocumentJsonlRow {
    queue_id: String,
    text: String,
    extractor: Option<String>,
    docling_profile: Option<String>,
    text_mime_type: Option<String>,
    source_sha256: String,
    extension: String,
}

#[derive(Debug, Serialize)]
struct DoclingDocumentCacheOutput {
    schema_version: &'static str,
    status: &'static str,
    queue_id: String,
    file_id: String,
    relative_path: String,
    extension: String,
    category: String,
    language: String,
    extraction_route: String,
    route_family: String,
    support_state: &'static str,
    source_sha256: String,
    source_hash_matched: JsonBool,
    extractor: String,
    docling_profile: String,
    text_mime_type: String,
    output_contract: &'static str,
    ocr_required: JsonBool,
    docling_document_executed: JsonBool,
    raw_content_extracted: JsonBool,
    raw_to_rdf_promotion_allowed: JsonBool,
    ontology_truth: JsonBool,
    review_status: &'static str,
    promotion_status: &'static str,
    text_char_count: usize,
    text_sha256: String,
    extracted_text: String,
}

#[derive(Debug, Serialize)]
struct DoclingDocumentCacheFailureOutput {
    schema_version: &'static str,
    status: &'static str,
    queue_id: String,
    file_id: String,
    relative_path: String,
    extension: String,
    category: String,
    language: String,
    extraction_route: String,
    route_family: String,
    support_state: &'static str,
    source_sha256: String,
    source_hash_matched: JsonBool,
    output_contract: &'static str,
    ocr_required: JsonBool,
    docling_document_executed: JsonBool,
    raw_content_extracted: JsonBool,
    raw_to_rdf_promotion_allowed: JsonBool,
    ontology_truth: JsonBool,
    text_char_count: usize,
    error: String,
}

#[derive(Debug, Serialize)]
struct DoclingDocumentCacheReceipt {
    schema_version: &'static str,
    extraction_executed: bool,
    raw_content_extracted: bool,
    document_seed_execution: EpistemeDoclingDocumentCacheBridgeReport,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
struct DoclingDocumentCacheWriteSummary {
    status_counts: BTreeMap<String, usize>,
    extractor_counts: BTreeMap<String, usize>,
    extension_counts: BTreeMap<String, usize>,
    total_text_chars: usize,
    succeeded_count: usize,
    failed_count: usize,
}

/// Build a dry-run bridge report without writing cache rows.
#[must_use]
pub fn skipped_docling_document_cache_bridge_report(
    document_results_jsonl: &Path,
    outputs_dir: &Path,
    receipt_path: &Path,
) -> EpistemeDoclingDocumentCacheBridgeReport {
    EpistemeDoclingDocumentCacheBridgeReport {
        schema_version: CACHE_RECEIPT_SCHEMA,
        skipped: true,
        passed: true,
        document_results_jsonl: display_path(document_results_jsonl),
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

/// Validate that selected tasks are supported by the Docling document route.
///
/// # Errors
///
/// Returns an error when any task extension is outside the supported
/// `pdf/docx/pptx/xlsx` set.
pub fn validate_docling_document_tasks(tasks: &[EpistemeCacheTask]) -> Result<()> {
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
        "Docling document cache selected unsupported document tasks: {}. Use a selection run/category that contains only pdf/docx/pptx/xlsx, or add a separate legacy conversion contract first.",
        unsupported.join(", ")
    )
}

/// Read Docling document cache tasks from the stable extraction `tasks.tsv`.
///
/// # Errors
///
/// Returns an error when the TSV is missing, has an unexpected header, or has
/// malformed task rows.
pub fn read_docling_document_tasks_tsv(path: &Path) -> Result<Vec<EpistemeCacheTask>> {
    read_tasks_tsv(path, "Docling document")
}

/// Write deterministic cache rows from analyzer Docling document JSONL output.
///
/// # Errors
///
/// Returns an error when the analyzer JSONL is malformed, contains unknown task
/// ids, or a planned output path escapes the run outputs directory.
pub fn write_docling_document_cache_outputs(
    tasks: &[EpistemeCacheTask],
    document_results_jsonl: &Path,
    run_dir: &Path,
    corpus_root: &Path,
) -> Result<EpistemeDoclingDocumentCacheBridgeReport> {
    let outputs_dir = run_dir.join("outputs");
    fs::create_dir_all(&outputs_dir)
        .with_context(|| format!("failed to create `{}`", outputs_dir.display()))?;
    let receipt_path = run_dir.join("document_cache_receipt.json");
    let rows = read_docling_document_jsonl(document_results_jsonl)?;
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
            "Docling document results reference unknown queue ids: {}",
            unknown.join(", ")
        );
    }

    let summary = write_docling_document_cache_task_outputs(tasks, &rows, run_dir, corpus_root)?;

    let report = EpistemeDoclingDocumentCacheBridgeReport {
        schema_version: CACHE_RECEIPT_SCHEMA,
        skipped: false,
        passed: summary.failed_count == 0,
        document_results_jsonl: display_path(document_results_jsonl),
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
    let receipt = DoclingDocumentCacheReceipt {
        schema_version: CACHE_RECEIPT_SCHEMA,
        extraction_executed: true,
        raw_content_extracted: report.succeeded_count > 0,
        document_seed_execution: report.clone(),
    };
    write_json(&receipt_path, &receipt)?;
    Ok(report)
}

fn write_docling_document_cache_task_outputs(
    tasks: &[EpistemeCacheTask],
    rows: &BTreeMap<String, DoclingDocumentJsonlRow>,
    run_dir: &Path,
    corpus_root: &Path,
) -> Result<DoclingDocumentCacheWriteSummary> {
    let mut summary = DoclingDocumentCacheWriteSummary::default();
    for task in tasks {
        write_one_docling_document_cache_output(task, rows, run_dir, corpus_root, &mut summary)?;
    }
    Ok(summary)
}

fn write_one_docling_document_cache_output(
    task: &EpistemeCacheTask,
    rows: &BTreeMap<String, DoclingDocumentJsonlRow>,
    run_dir: &Path,
    corpus_root: &Path,
    summary: &mut DoclingDocumentCacheWriteSummary,
) -> Result<()> {
    let extension = task_extension(task);
    increment(&mut summary.extension_counts, &extension);
    let output_path = resolve_run_output_path(run_dir, &task.planned_output_path, &task.queue_id)?;
    let source_hash = match build_task_context(task, corpus_root) {
        Ok(value) => value,
        Err(error) => {
            write_failure_output(&output_path, task, &extension, false, &error)?;
            increment(&mut summary.status_counts, "failed");
            summary.failed_count += 1;
            return Ok(());
        }
    };
    let Some(row) = rows.get(&task.queue_id) else {
        write_failure_output(
            &output_path,
            task,
            &extension,
            true,
            "Docling document result row is missing",
        )?;
        increment(&mut summary.status_counts, "failed");
        summary.failed_count += 1;
        return Ok(());
    };
    if let Err(error) = validate_result_row(task, row, &extension, &source_hash) {
        write_failure_output(&output_path, task, &extension, false, &error)?;
        increment(&mut summary.status_counts, "failed");
        summary.failed_count += 1;
        return Ok(());
    }
    let text = normalized_text(&row.text);
    if text.is_empty() {
        write_failure_output(
            &output_path,
            task,
            &extension,
            true,
            "Docling document result text is empty",
        )?;
        increment(&mut summary.status_counts, "failed");
        summary.failed_count += 1;
        return Ok(());
    }
    write_success_output(&output_path, task, &extension, row, &text, summary)
}

fn write_success_output(
    output_path: &Path,
    task: &EpistemeCacheTask,
    extension: &str,
    row: &DoclingDocumentJsonlRow,
    text: &str,
    summary: &mut DoclingDocumentCacheWriteSummary,
) -> Result<()> {
    let extractor = non_empty_or(row.extractor.as_deref(), "docling");
    let output = DoclingDocumentCacheOutput {
        schema_version: CACHE_OUTPUT_SCHEMA,
        status: "succeeded",
        queue_id: task.queue_id.clone(),
        file_id: task.file_id.clone(),
        relative_path: task.relative_path.clone(),
        extension: extension.to_string(),
        category: task.category.as_str().to_string(),
        language: task.language.clone(),
        extraction_route: task.extraction_route.clone(),
        route_family: route_family_for_task(task),
        support_state: "planned",
        source_sha256: task.source_sha256.clone(),
        source_hash_matched: true.into(),
        extractor: extractor.clone(),
        docling_profile: non_empty_or(row.docling_profile.as_deref(), "full"),
        text_mime_type: non_empty_or(row.text_mime_type.as_deref(), "text/markdown"),
        output_contract: OUTPUT_CONTRACT,
        ocr_required: false.into(),
        docling_document_executed: true.into(),
        raw_content_extracted: true.into(),
        raw_to_rdf_promotion_allowed: false.into(),
        ontology_truth: false.into(),
        review_status: "review_required",
        promotion_status: "blocked_pending_review",
        text_char_count: text.chars().count(),
        text_sha256: sha256_text(text),
        extracted_text: text.to_string(),
    };
    write_json(output_path, &output)?;
    increment(&mut summary.status_counts, "succeeded");
    increment(&mut summary.extractor_counts, &extractor);
    summary.total_text_chars += output.text_char_count;
    summary.succeeded_count += 1;
    Ok(())
}

fn read_docling_document_jsonl(path: &Path) -> Result<BTreeMap<String, DoclingDocumentJsonlRow>> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let mut rows = BTreeMap::new();
    for (index, line) in raw.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let row = serde_json::from_str::<DoclingDocumentJsonlRow>(line).with_context(|| {
            format!(
                "failed to parse Docling document JSONL line {} in `{}`",
                index + 1,
                path.display()
            )
        })?;
        let queue_id = row.queue_id.trim().to_string();
        if queue_id.is_empty() {
            anyhow::bail!("Docling document JSONL line {} missing queue_id", index + 1);
        }
        if rows.insert(queue_id.clone(), row).is_some() {
            anyhow::bail!("duplicate Docling document queue_id `{queue_id}`");
        }
    }
    Ok(rows)
}

fn validate_result_row(
    task: &EpistemeCacheTask,
    row: &DoclingDocumentJsonlRow,
    extension: &str,
    source_hash: &str,
) -> Result<(), String> {
    if task.extraction_route != EPISTEME_DOCLING_DOCUMENT_ROUTE {
        return Err(format!(
            "Docling document result targeted non-document route `{}` for `{}`",
            task.extraction_route, task.queue_id
        ));
    }
    if task.output_contract != OUTPUT_CONTRACT {
        return Err(format!(
            "Docling document task `{}` has unsupported output contract `{}`",
            task.queue_id, task.output_contract
        ));
    }
    if row.source_sha256 != task.source_sha256 || source_hash != task.source_sha256 {
        return Err(format!(
            "Docling document source sha256 drift for `{}`",
            task.queue_id
        ));
    }
    if row.extension != extension {
        return Err(format!(
            "Docling document extension mismatch for `{}`: result `{}` task `{extension}`",
            task.queue_id, row.extension
        ));
    }
    Ok(())
}

fn build_task_context(task: &EpistemeCacheTask, corpus_root: &Path) -> Result<String, String> {
    if task.extraction_route != EPISTEME_DOCLING_DOCUMENT_ROUTE {
        return Err(format!(
            "Docling document task targeted non-document route `{}`",
            task.extraction_route
        ));
    }
    if task.output_contract != OUTPUT_CONTRACT {
        return Err(format!(
            "Docling document task has unsupported output contract `{}`",
            task.output_contract
        ));
    }
    let source_path =
        resolve_existing_corpus_file(corpus_root, &task.relative_path, &task.queue_id)
            .map_err(|error| error.to_string())?;
    if !source_path.is_file() {
        return Err("source document is missing".to_string());
    }
    let source_hash = sha256_file(&source_path).map_err(|error| error.to_string())?;
    if source_hash != task.source_sha256 {
        return Err(format!("source sha256 drift for `{}`", task.queue_id));
    }
    Ok(source_hash)
}

fn write_failure_output(
    path: &Path,
    task: &EpistemeCacheTask,
    extension: &str,
    source_hash_matched: bool,
    error: &str,
) -> Result<()> {
    let output = DoclingDocumentCacheFailureOutput {
        schema_version: CACHE_OUTPUT_SCHEMA,
        status: "failed",
        queue_id: task.queue_id.clone(),
        file_id: task.file_id.clone(),
        relative_path: task.relative_path.clone(),
        extension: extension.to_string(),
        category: task.category.as_str().to_string(),
        language: task.language.clone(),
        extraction_route: task.extraction_route.clone(),
        route_family: route_family_for_task(task),
        support_state: "planned",
        source_sha256: task.source_sha256.clone(),
        source_hash_matched: source_hash_matched.into(),
        output_contract: OUTPUT_CONTRACT,
        ocr_required: false.into(),
        docling_document_executed: false.into(),
        raw_content_extracted: false.into(),
        raw_to_rdf_promotion_allowed: false.into(),
        ontology_truth: false.into(),
        text_char_count: 0,
        error: error.to_string(),
    };
    write_json(path, &output)
}

fn route_family_for_task(task: &EpistemeCacheTask) -> String {
    match (
        task.extraction_route.as_str(),
        task_extension(task).as_str(),
    ) {
        (EPISTEME_DOCLING_DOCUMENT_ROUTE, "pdf") => "pdf_document".to_string(),
        (EPISTEME_DOCLING_DOCUMENT_ROUTE, _) => "office_document".to_string(),
        _ => "document".to_string(),
    }
}
