//! Source-contract extraction run-plan DTOs and selectors.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use serde::Serialize;
use xiuxian_wendao_parsers::{EpistemeExtractionQueueRow, EpistemeFileRow, EpistemeSourceManifest};

use super::{
    EXTRACTION_QUEUE_TSV, EpistemeError, FILES_TSV, OUTPUT_CONTRACT, PENDING_STATUS,
    PLANNED_STATUS, RUN_PLAN_VALIDATION_MODE_CONTRACT_SHAPE_ONLY, RUN_SCHEMA_VERSION, count_by,
    extension_routes, read_files_tsv, read_queue_tsv, read_source_manifest, safe_run_id,
    source_contract_paths, validate_mapping_ledger, validate_queue_rows,
};

/// Raw DTO boundary and stringly state boundary for extraction planning input.
///
/// Request for deterministic Rust-owned extraction run planning.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeRunPlanRequest {
    /// Episteme repository root.
    pub episteme_root: PathBuf,
    /// Raw corpus root.
    pub corpus_root: PathBuf,
    /// Safe ASCII run id.
    pub run_id: String,
    /// Optional extraction route filter.
    pub route: Option<String>,
    /// Optional category filter.
    pub category: Option<String>,
    /// Maximum number of queue rows to select.
    pub limit: usize,
    /// Optional evidence-selection file ids that constrain queue planning.
    pub selected_file_ids: Option<Vec<String>>,
}

impl EpistemeRunPlanRequest {
    /// Create a request for Rust-owned episteme source-contract run planning.
    #[must_use]
    pub fn new(
        episteme_root: impl Into<PathBuf>,
        corpus_root: impl Into<PathBuf>,
        run_id: impl Into<String>,
    ) -> Self {
        Self {
            episteme_root: episteme_root.into(),
            corpus_root: corpus_root.into(),
            run_id: run_id.into(),
            route: None,
            category: None,
            limit: 12,
            selected_file_ids: None,
        }
    }

    /// Add an extraction route filter.
    #[must_use]
    pub fn with_route(mut self, route: impl Into<String>) -> Self {
        self.route = Some(route.into());
        self
    }

    /// Add a category filter.
    #[must_use]
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Set the selected queue row limit.
    #[must_use]
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Constrain planning to selected source file ids.
    #[must_use]
    pub fn with_selected_file_ids(mut self, file_ids: Vec<String>) -> Self {
        self.selected_file_ids = Some(file_ids);
        self
    }
}

/// Raw DTO boundary and stringly state boundary for extraction run tasks.
///
/// One selected episteme source-contract extraction run task.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct EpistemeRunTask {
    /// Queue row id.
    pub queue_id: String,
    /// Source file id.
    pub file_id: String,
    /// Source path relative to the corpus root.
    pub relative_path: String,
    /// Source category.
    pub category: String,
    /// Source language.
    pub language: String,
    /// Extraction route.
    pub extraction_route: String,
    /// Queue priority.
    pub priority: u32,
    /// Source SHA-256 copied from `files.tsv`.
    pub source_sha256: String,
    /// Planned local output path relative to a run directory.
    pub planned_output_path: String,
    /// Output contract.
    pub output_contract: String,
    /// Planned task status.
    pub status: String,
}

/// Raw DTO boundary and stringly state boundary for extraction run receipts.
///
/// Deterministic Rust-owned episteme source-contract run-plan receipt.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct EpistemeRunPlanReceipt {
    /// Receipt schema version.
    pub schema_version: &'static str,
    /// Safe ASCII run id.
    pub run_id: String,
    /// Optional route filter.
    pub route: Option<String>,
    /// Optional category filter.
    pub category: Option<String>,
    /// Selection limit.
    pub limit: usize,
    /// Evidence-selection file ids used to constrain this plan.
    pub selected_file_ids: Vec<String>,
    /// Total queue rows available.
    pub total_queue_rows: usize,
    /// Number of selected rows.
    pub selected_count: usize,
    /// Selected row counts by route.
    pub route_counts: BTreeMap<String, usize>,
    /// Selected row counts by category.
    pub category_counts: BTreeMap<String, usize>,
    /// Output contract.
    pub output_contract: String,
    /// Whether direct RDF promotion is allowed.
    pub raw_to_rdf_promotion_allowed: bool,
    /// Whether extraction ran during planning.
    pub extraction_executed: bool,
    /// Validation mode used during planning.
    pub validation_mode: &'static str,
    /// Selected tasks.
    pub tasks: Vec<EpistemeRunTask>,
}

/// Plan a episteme source-contract extraction run from the Rust backend boundary.
///
/// # Errors
///
/// Returns an error when the source contract is invalid, the run id is unsafe,
/// or no selectable queue rows match the request.
pub fn plan_episteme_extraction_run(
    request: &EpistemeRunPlanRequest,
) -> Result<EpistemeRunPlanReceipt, EpistemeError> {
    safe_run_id(&request.run_id)?;
    if request.limit == 0 {
        return Err(EpistemeError::EmptySelection);
    }

    let manifest = read_source_manifest(&request.episteme_root)?;
    let paths = source_contract_paths(&request.episteme_root)?;
    let corpus_dir = paths.corpus_dir(&request.episteme_root)?;
    let files = read_files_tsv(&corpus_dir.join(&manifest.files))?;
    let queue = read_queue_tsv(&corpus_dir.join(&manifest.extraction_queue))?;
    validate_extraction_plan_contract_shape(
        &request.episteme_root,
        &request.corpus_root,
        &manifest,
        &files,
        &queue,
    )?;
    let files_by_id = files
        .iter()
        .map(|row| (row.file_id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let selected = select_queue_rows(request, &queue)?;

    if selected.is_empty() {
        return Err(EpistemeError::EmptySelection);
    }

    let tasks = selected
        .iter()
        .map(|row| {
            let source = files_by_id.get(row.file_id.as_str()).ok_or_else(|| {
                EpistemeError::InvalidContract(vec![format!(
                    "queue row references unknown file_id: {}",
                    row.file_id
                )])
            })?;
            Ok(EpistemeRunTask {
                queue_id: row.queue_id.clone(),
                file_id: row.file_id.clone(),
                relative_path: row.relative_path.clone(),
                category: row.category.clone(),
                language: row.language.clone(),
                extraction_route: row.extraction_route.clone(),
                priority: row.priority,
                source_sha256: source.sha256.clone(),
                planned_output_path: format!("outputs/{}.json", row.queue_id),
                output_contract: OUTPUT_CONTRACT.to_string(),
                status: PLANNED_STATUS.to_string(),
            })
        })
        .collect::<Result<Vec<_>, EpistemeError>>()?;

    Ok(EpistemeRunPlanReceipt {
        schema_version: RUN_SCHEMA_VERSION,
        run_id: request.run_id.clone(),
        route: request.route.clone(),
        category: request.category.clone(),
        limit: request.limit,
        selected_file_ids: request.selected_file_ids.clone().unwrap_or_default(),
        total_queue_rows: queue.len(),
        selected_count: tasks.len(),
        route_counts: count_by(tasks.iter().map(|task| task.extraction_route.as_str())),
        category_counts: count_by(tasks.iter().map(|task| task.category.as_str())),
        output_contract: OUTPUT_CONTRACT.to_string(),
        raw_to_rdf_promotion_allowed: false,
        extraction_executed: false,
        validation_mode: RUN_PLAN_VALIDATION_MODE_CONTRACT_SHAPE_ONLY,
        tasks,
    })
}

fn validate_extraction_plan_contract_shape(
    episteme_root: &Path,
    corpus_root: &Path,
    manifest: &EpistemeSourceManifest,
    files: &[EpistemeFileRow],
    queue: &[EpistemeExtractionQueueRow],
) -> Result<(), EpistemeError> {
    let mut errors = Vec::new();
    validate_mapping_ledger(episteme_root, &mut errors)?;
    errors.extend(validate_contract_shape_only(
        corpus_root,
        manifest,
        files,
        queue,
    ));
    if errors.is_empty() {
        Ok(())
    } else {
        Err(EpistemeError::InvalidContract(errors))
    }
}

fn validate_contract_shape_only(
    corpus_root: &Path,
    manifest: &EpistemeSourceManifest,
    files: &[EpistemeFileRow],
    queue: &[EpistemeExtractionQueueRow],
) -> Vec<String> {
    let mut errors = Vec::new();
    if manifest.copy_raw_files {
        errors.push("source manifest copy_raw_files must be false".to_string());
    }
    if manifest.raw_to_rdf_promotion_allowed {
        errors.push("source manifest raw_to_rdf_promotion_allowed must be false".to_string());
    }
    if manifest.primary_language != "zh-CN" {
        errors.push("source manifest primary_language must be zh-CN".to_string());
    }
    if manifest.files != FILES_TSV {
        errors.push("source manifest files must be files.tsv".to_string());
    }
    if manifest.extraction_queue != EXTRACTION_QUEUE_TSV {
        errors.push("source manifest extraction_queue must be extraction_queue.tsv".to_string());
    }
    if !corpus_root.is_dir() {
        errors.push(format!(
            "corpus root does not exist: {}",
            corpus_root.display()
        ));
        return errors;
    }

    let extension_routes = extension_routes(manifest);
    let mut file_ids = BTreeSet::new();
    let mut file_paths = BTreeSet::new();
    for (index, row) in files.iter().enumerate() {
        let row_number = index + 2;
        if !file_ids.insert(row.file_id.as_str()) {
            errors.push(format!(
                "duplicate file_id at row {row_number}: {}",
                row.file_id
            ));
        }
        if !file_paths.insert(row.relative_path.clone()) {
            errors.push(format!(
                "duplicate relative_path at row {row_number}: {}",
                row.relative_path
            ));
        }
        if row.language != manifest.primary_language {
            errors.push(format!(
                "row {row_number} language must be {}",
                manifest.primary_language
            ));
        }
        match extension_routes.get(row.extension.as_str()) {
            Some(route) if route == &row.extraction_route => {}
            Some(route) => errors.push(format!(
                "row {row_number} extraction route should be {route}: {}",
                row.relative_path
            )),
            None => errors.push(format!(
                "row {row_number} unknown extension: {}",
                row.extension
            )),
        }
        if row.category.is_empty() {
            errors.push(format!("row {row_number} missing category"));
        }
    }

    validate_queue_rows(queue, files, &mut errors);
    errors
}

fn select_queue_rows<'a>(
    request: &EpistemeRunPlanRequest,
    queue: &'a [EpistemeExtractionQueueRow],
) -> Result<Vec<&'a EpistemeExtractionQueueRow>, EpistemeError> {
    match &request.selected_file_ids {
        Some(file_ids) => select_queue_rows_from_file_ids(request, queue, file_ids),
        None => Ok(queue
            .iter()
            .filter(|row| queue_row_matches_request(request, row))
            .take(request.limit)
            .collect::<Vec<_>>()),
    }
}

fn select_queue_rows_from_file_ids<'a>(
    request: &EpistemeRunPlanRequest,
    queue: &'a [EpistemeExtractionQueueRow],
    file_ids: &[String],
) -> Result<Vec<&'a EpistemeExtractionQueueRow>, EpistemeError> {
    validate_selected_file_ids(file_ids)?;
    if file_ids.len() > request.limit {
        return Err(EpistemeError::InvalidContract(vec![format!(
            "selection contains {} file ids but run-plan limit is {}",
            file_ids.len(),
            request.limit
        )]));
    }
    let mut selected = Vec::with_capacity(file_ids.len());
    let mut errors = Vec::new();
    for file_id in file_ids {
        match queue.iter().find(|row| {
            row.file_id == *file_id
                && row.status == PENDING_STATUS
                && queue_row_matches_filters(request, row)
        }) {
            Some(row) => selected.push(row),
            None => errors.push(format!(
                "selected file_id has no plannable pending queue row: {file_id}"
            )),
        }
    }
    if errors.is_empty() {
        Ok(selected)
    } else {
        Err(EpistemeError::InvalidContract(errors))
    }
}

fn validate_selected_file_ids(file_ids: &[String]) -> Result<(), EpistemeError> {
    if file_ids.is_empty() {
        return Err(EpistemeError::EmptySelection);
    }
    let mut seen = BTreeSet::new();
    let duplicates = file_ids
        .iter()
        .filter(|file_id| !seen.insert(file_id.as_str()))
        .map(|file_id| format!("duplicate selected file_id: {file_id}"))
        .collect::<Vec<_>>();
    if duplicates.is_empty() {
        Ok(())
    } else {
        Err(EpistemeError::InvalidContract(duplicates))
    }
}

fn queue_row_matches_request(
    request: &EpistemeRunPlanRequest,
    row: &EpistemeExtractionQueueRow,
) -> bool {
    row.status == PENDING_STATUS && queue_row_matches_filters(request, row)
}

fn queue_row_matches_filters(
    request: &EpistemeRunPlanRequest,
    row: &EpistemeExtractionQueueRow,
) -> bool {
    request
        .route
        .as_ref()
        .is_none_or(|route| row.extraction_route == *route)
        && request
            .category
            .as_ref()
            .is_none_or(|category| row.category == *category)
}
