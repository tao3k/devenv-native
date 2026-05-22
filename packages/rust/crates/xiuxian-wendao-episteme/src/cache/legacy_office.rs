//! Legacy Office conversion admission and execution for Episteme source contracts.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use serde::Serialize;

use super::{
    materialization_support::{display_path, increment, sha256_file, write_json},
    path::{resolve_existing_corpus_file, resolve_run_output_path},
    task::{EpistemeCacheTask, read_tasks_tsv, task_extension},
};

/// Source-contract extraction route for legacy Office conversion candidates.
pub const EPISTEME_LEGACY_OFFICE_DOCUMENT_ROUTE: &str = "legacy_office_document_evidence";
/// Receipt filename emitted by the legacy Office converter runner.
pub const EPISTEME_LEGACY_OFFICE_CONVERSION_RECEIPT_JSON: &str =
    "legacy_office_conversion_receipt.json";
/// Wrapper schema for the Studio legacy Office conversion command report.
pub const EPISTEME_LEGACY_OFFICE_CONVERSION_WRAPPER_SCHEMA: &str =
    "xiuxian_wendao.episteme_legacy_office_conversion_execution.v1";

const CONVERSION_SCHEMA: &str = "xiuxian_wendao.episteme_legacy_office_conversion.v1";
const CONVERSION_RECEIPT_SCHEMA: &str =
    "xiuxian_wendao.episteme_legacy_office_conversion_receipt.v1";
const OUTPUT_CONTRACT: &str = "legacy_office_conversion_only_no_rdf_promotion";
const SUPPORTED_EXTENSIONS: [&str; 3] = ["doc", "ppt", "xls"];

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(transparent)]
struct JsonBool(bool);

impl From<bool> for JsonBool {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

/// Request for executing a legacy Office converter over admitted tasks.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeLegacyOfficeConversionRequest {
    /// Explicit converter executable. It is called as
    /// `converter <source-path> <target-path>`.
    pub converter_path: PathBuf,
    /// When true, validate tasks and paths but do not execute the converter.
    pub dry_run: bool,
}

impl EpistemeLegacyOfficeConversionRequest {
    /// Create a conversion request with an explicit converter binary path.
    #[must_use]
    pub fn new(converter_path: impl Into<PathBuf>) -> Self {
        Self {
            converter_path: converter_path.into(),
            dry_run: false,
        }
    }

    /// Configure dry-run mode.
    #[must_use]
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }
}

/// Report emitted by a legacy Office conversion execution pass.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeLegacyOfficeConversionReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Whether converter execution was skipped by dry-run mode.
    pub skipped: bool,
    /// Whether every selected task succeeded or was skipped in dry-run mode.
    pub passed: bool,
    /// Converter executable path.
    pub converter_path: String,
    /// Run directory path.
    pub run_dir: String,
    /// Converted artifact directory.
    pub converted_dir: String,
    /// Receipt JSON path.
    pub receipt_path: String,
    /// Number of planned tasks.
    pub attempted_count: usize,
    /// Number of successful conversions.
    pub succeeded_count: usize,
    /// Number of failed conversions.
    pub failed_count: usize,
    /// Number of dry-run skipped tasks.
    pub skipped_count: usize,
    /// Count by output status.
    pub status_counts: BTreeMap<String, usize>,
    /// Count by source extension.
    pub extension_counts: BTreeMap<String, usize>,
    /// Converted rows are evidence only and cannot be promoted directly to RDF.
    pub raw_to_rdf_promotion_allowed: bool,
}

#[derive(Debug, Serialize)]
struct LegacyOfficeConversionOutput {
    schema_version: &'static str,
    status: &'static str,
    queue_id: String,
    file_id: String,
    relative_path: String,
    extension: String,
    category: String,
    language: String,
    extraction_route: String,
    source_sha256: String,
    source_hash_matched: JsonBool,
    converter_path: String,
    converted_artifact_path: Option<String>,
    converted_extension: Option<String>,
    converted_sha256: Option<String>,
    output_contract: &'static str,
    conversion_executed: JsonBool,
    raw_to_rdf_promotion_allowed: JsonBool,
    ontology_truth: JsonBool,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct LegacyOfficeConversionReceipt {
    schema_version: &'static str,
    conversion_executed: bool,
    raw_to_rdf_promotion_allowed: bool,
    legacy_office_conversion: EpistemeLegacyOfficeConversionReport,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
struct LegacyOfficeConversionSummary {
    status_counts: BTreeMap<String, usize>,
    extension_counts: BTreeMap<String, usize>,
    succeeded_count: usize,
    failed_count: usize,
    skipped_count: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct LegacyOfficeConversionTarget {
    source_extension: String,
    converted_extension: String,
    converted_artifact_path: PathBuf,
}

#[derive(Debug)]
enum VerifiedLegacyOfficeSource {
    Ready(PathBuf),
    Drift,
    Failed(Box<LegacyOfficeConversionOutput>),
}

/// Read legacy Office conversion tasks from the stable extraction `tasks.tsv`.
///
/// # Errors
///
/// Returns an error when the TSV is missing, has an unexpected header, or has
/// malformed task rows.
pub fn read_legacy_office_conversion_tasks_tsv(path: &Path) -> Result<Vec<EpistemeCacheTask>> {
    read_tasks_tsv(path, "Legacy Office conversion")
}

/// Validate that selected tasks are legacy Office conversion candidates.
///
/// # Errors
///
/// Returns an error when any task uses a non-legacy extension or a route other
/// than `legacy_office_document_evidence`.
pub fn validate_legacy_office_conversion_tasks(tasks: &[EpistemeCacheTask]) -> Result<()> {
    let invalid = tasks
        .iter()
        .filter_map(invalid_legacy_office_task)
        .collect::<Vec<_>>();
    if invalid.is_empty() {
        return Ok(());
    }
    anyhow::bail!(
        "Legacy Office conversion selected invalid tasks: {}. Select only doc/ppt/xls rows routed through {EPISTEME_LEGACY_OFFICE_DOCUMENT_ROUTE}.",
        invalid.join(", ")
    )
}

/// Execute a legacy Office conversion run using an explicit converter binary.
///
/// The converter is called once per task as `converter <source-path>
/// <target-path>`. Converted artifacts stay under the run directory and remain
/// promotion-blocked evidence.
///
/// # Errors
///
/// Returns an error when tasks are invalid, output paths escape the run
/// directory, source paths escape the corpus root, or receipt/output JSON cannot
/// be written.
pub fn convert_legacy_office_tasks(
    tasks: &[EpistemeCacheTask],
    run_dir: &Path,
    corpus_root: &Path,
    request: &EpistemeLegacyOfficeConversionRequest,
) -> Result<EpistemeLegacyOfficeConversionReport> {
    validate_legacy_office_conversion_tasks(tasks)?;
    let converted_dir = run_dir.join("outputs/converted");
    fs::create_dir_all(&converted_dir)
        .with_context(|| format!("failed to create `{}`", converted_dir.display()))?;
    let receipt_path = run_dir.join(EPISTEME_LEGACY_OFFICE_CONVERSION_RECEIPT_JSON);
    let mut summary = LegacyOfficeConversionSummary::default();

    for task in tasks {
        let output = convert_one_legacy_office_task(task, run_dir, corpus_root, request)?;
        increment(&mut summary.status_counts, output.status);
        increment(&mut summary.extension_counts, task_extension(task).as_str());
        match output.status {
            "succeeded" => summary.succeeded_count += 1,
            "skipped" => summary.skipped_count += 1,
            _ => summary.failed_count += 1,
        }
        let output_path =
            resolve_run_output_path(run_dir, &task.planned_output_path, &task.queue_id)?;
        write_json(&output_path, &output)?;
    }

    let report = EpistemeLegacyOfficeConversionReport {
        schema_version: CONVERSION_RECEIPT_SCHEMA,
        skipped: request.dry_run,
        passed: summary.failed_count == 0,
        converter_path: display_path(&request.converter_path),
        run_dir: display_path(run_dir),
        converted_dir: display_path(&converted_dir),
        receipt_path: display_path(&receipt_path),
        attempted_count: tasks.len(),
        succeeded_count: summary.succeeded_count,
        failed_count: summary.failed_count,
        skipped_count: summary.skipped_count,
        status_counts: summary.status_counts,
        extension_counts: summary.extension_counts,
        raw_to_rdf_promotion_allowed: false,
    };
    let receipt = LegacyOfficeConversionReceipt {
        schema_version: CONVERSION_RECEIPT_SCHEMA,
        conversion_executed: !request.dry_run,
        raw_to_rdf_promotion_allowed: false,
        legacy_office_conversion: report.clone(),
    };
    write_json(&receipt_path, &receipt)?;
    Ok(report)
}

fn invalid_legacy_office_task(task: &EpistemeCacheTask) -> Option<String> {
    let extension = task_extension(task);
    let legacy_extension = SUPPORTED_EXTENSIONS.contains(&extension.as_str());
    let legacy_route = task.extraction_route == EPISTEME_LEGACY_OFFICE_DOCUMENT_ROUTE;
    (!legacy_extension || !legacy_route).then(|| {
        format!(
            "{} (route={}, extension={extension})",
            task.queue_id, task.extraction_route
        )
    })
}

fn convert_one_legacy_office_task(
    task: &EpistemeCacheTask,
    run_dir: &Path,
    corpus_root: &Path,
    request: &EpistemeLegacyOfficeConversionRequest,
) -> Result<LegacyOfficeConversionOutput> {
    let target = legacy_office_conversion_target(run_dir, task)?;
    let source_path = match verified_legacy_office_source(task, corpus_root, request, &target) {
        VerifiedLegacyOfficeSource::Ready(path) => path,
        VerifiedLegacyOfficeSource::Drift => {
            return Ok(failed_output(
                task,
                request,
                false,
                false,
                Some(display_path(&target.converted_artifact_path)),
                Some(target.converted_extension.clone()),
                "source sha256 drift".to_string(),
            ));
        }
        VerifiedLegacyOfficeSource::Failed(output) => return Ok(*output),
    };

    if request.dry_run {
        return Ok(skipped_output(task, request, &target));
    }

    if let Err(error) = run_legacy_office_converter(request, &source_path, &target) {
        return Ok(failed_output(
            task,
            request,
            true,
            true,
            Some(display_path(&target.converted_artifact_path)),
            Some(target.converted_extension.clone()),
            error,
        ));
    }
    let converted_sha256 = sha256_file(&target.converted_artifact_path)?;
    Ok(succeeded_output(task, request, &target, converted_sha256))
}

fn legacy_office_conversion_target(
    run_dir: &Path,
    task: &EpistemeCacheTask,
) -> Result<LegacyOfficeConversionTarget> {
    let source_extension = task_extension(task);
    let converted_extension = converted_extension_for(source_extension.as_str())?.to_string();
    let converted_artifact_path =
        converted_artifact_path(run_dir, task, converted_extension.as_str())?;
    Ok(LegacyOfficeConversionTarget {
        source_extension,
        converted_extension,
        converted_artifact_path,
    })
}

fn verified_legacy_office_source(
    task: &EpistemeCacheTask,
    corpus_root: &Path,
    request: &EpistemeLegacyOfficeConversionRequest,
    target: &LegacyOfficeConversionTarget,
) -> VerifiedLegacyOfficeSource {
    let source_path =
        match resolve_existing_corpus_file(corpus_root, &task.relative_path, &task.queue_id) {
            Ok(path) => path,
            Err(error) => {
                return VerifiedLegacyOfficeSource::Failed(Box::new(failed_output(
                    task,
                    request,
                    false,
                    false,
                    Some(display_path(&target.converted_artifact_path)),
                    Some(target.converted_extension.clone()),
                    error.to_string(),
                )));
            }
        };
    let source_sha256 = match sha256_file(&source_path) {
        Ok(hash) => hash,
        Err(error) => {
            return VerifiedLegacyOfficeSource::Failed(Box::new(failed_output(
                task,
                request,
                false,
                false,
                Some(display_path(&target.converted_artifact_path)),
                Some(target.converted_extension.clone()),
                error.to_string(),
            )));
        }
    };
    if source_sha256 == task.source_sha256 {
        VerifiedLegacyOfficeSource::Ready(source_path)
    } else {
        VerifiedLegacyOfficeSource::Drift
    }
}

fn run_legacy_office_converter(
    request: &EpistemeLegacyOfficeConversionRequest,
    source_path: &Path,
    target: &LegacyOfficeConversionTarget,
) -> Result<(), String> {
    let output = Command::new(&request.converter_path)
        .arg(source_path)
        .arg(&target.converted_artifact_path)
        .output()
        .map_err(|error| {
            format!(
                "failed to run legacy Office converter `{}`: {error}",
                request.converter_path.display(),
            )
        })?;
    if !output.status.success() {
        return Err(converter_error(&output.stderr));
    }
    if target.converted_artifact_path.is_file() {
        Ok(())
    } else {
        Err("converter did not produce the target artifact".to_string())
    }
}

fn skipped_output(
    task: &EpistemeCacheTask,
    request: &EpistemeLegacyOfficeConversionRequest,
    target: &LegacyOfficeConversionTarget,
) -> LegacyOfficeConversionOutput {
    successful_output(task, request, target, "skipped", false, None)
}

fn succeeded_output(
    task: &EpistemeCacheTask,
    request: &EpistemeLegacyOfficeConversionRequest,
    target: &LegacyOfficeConversionTarget,
    converted_sha256: String,
) -> LegacyOfficeConversionOutput {
    successful_output(
        task,
        request,
        target,
        "succeeded",
        true,
        Some(converted_sha256),
    )
}

fn successful_output(
    task: &EpistemeCacheTask,
    request: &EpistemeLegacyOfficeConversionRequest,
    target: &LegacyOfficeConversionTarget,
    status: &'static str,
    conversion_executed: bool,
    converted_sha256: Option<String>,
) -> LegacyOfficeConversionOutput {
    LegacyOfficeConversionOutput {
        schema_version: CONVERSION_SCHEMA,
        status,
        queue_id: task.queue_id.clone(),
        file_id: task.file_id.clone(),
        relative_path: task.relative_path.clone(),
        extension: target.source_extension.clone(),
        category: task.category.as_str().to_string(),
        language: task.language.clone(),
        extraction_route: task.extraction_route.clone(),
        source_sha256: task.source_sha256.clone(),
        source_hash_matched: true.into(),
        converter_path: display_path(&request.converter_path),
        converted_artifact_path: Some(display_path(&target.converted_artifact_path)),
        converted_extension: Some(target.converted_extension.clone()),
        converted_sha256,
        output_contract: OUTPUT_CONTRACT,
        conversion_executed: conversion_executed.into(),
        raw_to_rdf_promotion_allowed: false.into(),
        ontology_truth: false.into(),
        error: None,
    }
}

fn converted_artifact_path(
    run_dir: &Path,
    task: &EpistemeCacheTask,
    converted_extension: &str,
) -> Result<PathBuf> {
    let output_path = format!(
        "outputs/converted/{}.{}",
        safe_file_token(task.queue_id.as_str())?,
        converted_extension
    );
    resolve_run_output_path(run_dir, output_path.as_str(), &task.queue_id)
}

fn safe_file_token(value: &str) -> Result<&str> {
    let safe = !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'));
    if safe {
        Ok(value)
    } else {
        anyhow::bail!("legacy Office conversion queue_id `{value}` is not safe for artifact naming")
    }
}

fn converted_extension_for(extension: &str) -> Result<&'static str> {
    match extension {
        "doc" => Ok("docx"),
        "ppt" => Ok("pptx"),
        "xls" => Ok("xlsx"),
        _ => anyhow::bail!("unsupported legacy Office extension for conversion: {extension}"),
    }
}

fn failed_output(
    task: &EpistemeCacheTask,
    request: &EpistemeLegacyOfficeConversionRequest,
    conversion_executed: bool,
    source_hash_matched: bool,
    converted_artifact_path: Option<String>,
    converted_extension: Option<String>,
    error: String,
) -> LegacyOfficeConversionOutput {
    LegacyOfficeConversionOutput {
        schema_version: CONVERSION_SCHEMA,
        status: "failed",
        queue_id: task.queue_id.clone(),
        file_id: task.file_id.clone(),
        relative_path: task.relative_path.clone(),
        extension: task_extension(task),
        category: task.category.as_str().to_string(),
        language: task.language.clone(),
        extraction_route: task.extraction_route.clone(),
        source_sha256: task.source_sha256.clone(),
        source_hash_matched: source_hash_matched.into(),
        converter_path: display_path(&request.converter_path),
        converted_artifact_path,
        converted_extension,
        converted_sha256: None,
        output_contract: OUTPUT_CONTRACT,
        conversion_executed: conversion_executed.into(),
        raw_to_rdf_promotion_allowed: false.into(),
        ontology_truth: false.into(),
        error: Some(error),
    }
}

fn converter_error(stderr: &[u8]) -> String {
    let message = String::from_utf8_lossy(stderr).trim().to_string();
    if message.is_empty() {
        "converter exited with a non-zero status".to_string()
    } else {
        message
    }
}
