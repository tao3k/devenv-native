use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use walkdir::WalkDir;
use xiuxian_wendao_parsers::{
    EpistemeExtractionQueueRow, EpistemeFileRow, EpistemeMappingLedgerValidation,
    EpistemeSourceContractParseError, EpistemeSourceManifest, parse_episteme_extraction_queue_tsv,
    parse_episteme_files_tsv, parse_episteme_source_manifest_toml,
    validate_episteme_mapping_ledger_org,
};

mod config;
mod evidence;
mod read_model;
mod registry;
mod selection;
mod structure;
mod write;

pub use config::{EpistemeRuntimeConfig, load_episteme_runtime_config};
pub use evidence::{
    EpistemeEvidenceByteSizeStatus, EpistemeEvidenceReadReport, EpistemeEvidenceReadRequest,
    EpistemeEvidenceReadValidationMode, EpistemeEvidenceSha256Status,
    EpistemeEvidenceSourceAvailability, EpistemeEvidenceSourceRef, EpistemeEvidenceTextPreview,
    read_episteme_evidence,
};
#[cfg(feature = "julia")]
pub use read_model::build_episteme_wendaograph_quality_request_batches;
pub use read_model::{
    EpistemeReadModelMaterialization, EpistemeReadModelRequest, EpistemeReadModelTable,
    materialize_episteme_read_model_seed,
    materialize_episteme_read_model_seed_with_validation_hash_cache,
    materialize_episteme_registry_reference_graph_read_model_seed,
    validate_episteme_read_model_relation_endpoints,
};
pub use registry::{
    EpistemeRegistryEntry, EpistemeRegistryError, EpistemeRegistryLoadReceipt,
    EpistemeRegistryReferenceGraphEntry, EpistemeRegistryReferenceGraphLink,
    EpistemeRegistryReferenceGraphReceipt, LoadedEpistemeRegistryEntry, LoadedEpistemeSourceKind,
    load_episteme_registry_entries, load_episteme_registry_entries_with_mode,
    validate_episteme_registry_reference_graph,
};
pub use selection::{
    EpistemeEvidenceSelectionPlanRequest, EpistemeEvidenceSelectionReceipt,
    EpistemeEvidenceSelectionRow, EpistemeEvidenceSelectionValidationMode,
    EpistemeEvidenceSelectionWriteReport, read_episteme_evidence_selection_file_ids,
    write_episteme_evidence_selection_plan,
};
pub use structure::{
    EpistemeStructureTocReceipt, EpistemeStructureTocRequest, EpistemeStructureTocValidationMode,
    EpistemeStructureTocWriteReport, write_episteme_structure_toc,
};
pub use write::{EpistemeRunPlanWriteReport, write_episteme_extraction_run_plan};

const ONTOLOGY_MANIFEST_RELATIVE_PATH: &str = "ontology/manifest.toml";
const FILES_TSV: &str = "files.tsv";
const EXTRACTION_QUEUE_TSV: &str = "extraction_queue.tsv";
const OUTPUT_CONTRACT: &str = "cache_only_no_rdf_promotion";
const PENDING_STATUS: &str = "pending";
const PLANNED_STATUS: &str = "planned";
const RUN_SCHEMA_VERSION: &str = "xiuxian_wendao.episteme_extraction_run_plan.v1";
const RUN_PLAN_VALIDATION_MODE_CONTRACT_SHAPE_ONLY: &str = "contract_shape_only";
const VALIDATION_SCHEMA_VERSION: &str = "xiuxian_wendao.episteme_source_contract_validation.v1";
const VALIDATION_HASH_CACHE_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_source_contract_validation_hash_cache.v1";
const VALIDATION_HASH_CACHE_REPORT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_source_contract_validation_hash_cache_report.v1";

#[derive(Debug, Deserialize)]
struct EpistemeOntologyManifest {
    #[serde(default)]
    active_source_contract: Option<EpistemeActiveSourceContract>,
    #[serde(default)]
    domains: Vec<EpistemeDomainManifest>,
}

#[derive(Debug, Deserialize)]
struct EpistemeActiveSourceContract {
    domain_id: String,
    source_manifest: String,
    mapping_ledger: String,
}

#[derive(Debug, Deserialize)]
struct EpistemeDomainManifest {
    id: String,
    #[serde(default)]
    source_manifests: Vec<String>,
    #[serde(default)]
    mapping_ledgers: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct EpistemeSourceContractPaths {
    domain_id: String,
    source_manifest_relative_path: String,
    mapping_ledger_relative_path: String,
}

impl EpistemeSourceContractPaths {
    fn source_manifest_path(&self, episteme_root: &Path) -> PathBuf {
        episteme_root.join(&self.source_manifest_relative_path)
    }

    fn mapping_ledger_path(&self, episteme_root: &Path) -> PathBuf {
        episteme_root.join(&self.mapping_ledger_relative_path)
    }

    pub(super) fn source_manifest_relative_path(&self) -> &str {
        self.source_manifest_relative_path.as_str()
    }

    pub(super) fn mapping_ledger_relative_path(&self) -> &str {
        self.mapping_ledger_relative_path.as_str()
    }

    pub(super) fn domain_id(&self) -> &str {
        self.domain_id.as_str()
    }

    pub(super) fn corpus_dir(&self, episteme_root: &Path) -> Result<PathBuf, EpistemeError> {
        let manifest_relative_path = Path::new(self.source_manifest_relative_path.as_str());
        let Some(relative_dir) = manifest_relative_path.parent() else {
            return Err(EpistemeError::InvalidEpistemeManifest(format!(
                "source manifest path must have a parent directory: {}",
                self.source_manifest_relative_path
            )));
        };
        Ok(episteme_root.join(relative_dir))
    }

    pub(super) fn corpus_relative_path(&self, file_name: &str) -> String {
        let manifest_relative_path = Path::new(self.source_manifest_relative_path.as_str());
        manifest_relative_path
            .parent()
            .map_or_else(PathBuf::new, Path::to_path_buf)
            .join(file_name)
            .to_string_lossy()
            .replace('\\', "/")
    }
}

/// Return the configured corpus-root environment variable for an episteme.
///
/// The value is read from the source manifest selected by
/// `ontology/manifest.toml`; it is not hardcoded by Rust.
///
/// # Errors
///
/// Returns an error when the episteme config or selected source manifest cannot
/// be read or parsed.
pub fn configured_episteme_corpus_root_env(
    episteme_root: impl AsRef<Path>,
) -> Result<String, EpistemeError> {
    Ok(read_source_manifest(episteme_root.as_ref())?.corpus_root_env)
}

/// Error returned by Rust-owned episteme source-contract source planning.
#[derive(Debug, Error)]
pub enum EpistemeError {
    /// A file or directory could not be accessed.
    #[error("failed to access `{path}`: {source}")]
    Io {
        /// Path that failed.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
    /// A run-plan JSON file could not be serialized.
    #[error("failed to serialize episteme source-contract run-plan JSON `{path}`: {source}")]
    Json {
        /// Path that failed.
        path: PathBuf,
        /// Underlying JSON error.
        #[source]
        source: serde_json::Error,
    },
    /// A parser-owned source-contract input is malformed.
    #[error("failed to parse episteme source-contract source contract `{path}`: {source}")]
    Parse {
        /// Source contract path.
        path: PathBuf,
        /// Parser-owned parse error.
        #[source]
        source: EpistemeSourceContractParseError,
    },
    /// The episteme source-contract ontology manifest is malformed.
    #[error("failed to parse episteme source-contract manifest `{path}`: {source}")]
    EpistemeManifestToml {
        /// Manifest path.
        path: PathBuf,
        /// Underlying TOML error.
        #[source]
        source: toml::de::Error,
    },
    /// The episteme config cannot select one source contract.
    #[error("episteme source-contract manifest is invalid: {0}")]
    InvalidEpistemeManifest(String),
    /// The source contract is not valid, so planning cannot proceed.
    #[error("episteme source-contract source contract is invalid: {0:?}")]
    InvalidContract(Vec<String>),
    /// The requested run id is unsafe.
    #[error("invalid run id `{0}`; use ASCII letters, digits, '.', '_', or '-'")]
    InvalidRunId(String),
    /// No queue rows matched the request.
    #[error("no extraction queue rows matched the requested filters")]
    EmptySelection,
    /// The episteme source-contract read-model seed could not be materialized.
    #[error("failed to materialize episteme source-contract read-model seed: {0}")]
    ReadModel(String),
}

/// Source-contract validation report emitted by the Rust backend boundary.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct EpistemeValidationReport {
    /// Report schema version.
    pub schema_version: &'static str,
    /// True when no errors were found.
    pub passed: bool,
    /// Validation errors.
    pub errors: Vec<String>,
    /// Number of file rows loaded from `files.tsv`.
    pub files_tsv_rows: usize,
    /// Number of queue rows loaded from `extraction_queue.tsv`.
    pub extraction_queue_rows: usize,
    /// Primary language from the source manifest.
    pub primary_language: String,
    /// Corpus root env var name from the source manifest.
    pub corpus_root_env: String,
    /// Whether raw rows may be promoted directly to RDF truth.
    pub raw_to_rdf_promotion_allowed: bool,
    /// Typed Org authoring section count from the mapping ledger.
    pub mapping_ledger_sections: usize,
    /// Schema-governed reasoning property record count from the mapping ledger.
    pub mapping_ledger_reasoning_property_records: usize,
}

/// Report for opt-in episteme source-contract validation hash-cache usage.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct EpistemeValidationHashCacheReport {
    /// Report schema version.
    pub schema_version: &'static str,
    /// Cache file path used for this validation run.
    pub cache_path: PathBuf,
    /// Number of cache entries loaded from disk.
    pub entries_loaded: usize,
    /// True when the cache file existed but could not be parsed.
    pub malformed_cache: bool,
    /// Rows whose hash was accepted from cache.
    pub hash_cache_hits: usize,
    /// Rows that required a full file hash.
    pub hash_cache_misses: usize,
    /// Existing cache entries rejected by metadata or expected hash mismatch.
    pub stale_entries: usize,
    /// Entries written back after successful full hash checks.
    pub entries_written: usize,
}

/// Validate a episteme source-contract source contract with an opt-in hash cache.
///
/// The cache is an accelerator only. Manifest and TSV parsing still run every
/// time, and cache entries are accepted only when relative path, byte size,
/// modified time, and expected SHA-256 all match.
///
/// # Errors
///
/// Returns an error when the manifest or TSV files cannot be read or parsed,
/// source files cannot be inspected, or the cache file cannot be written.
pub fn validate_episteme_source_contract_with_hash_cache(
    episteme_root: impl AsRef<Path>,
    corpus_root: impl AsRef<Path>,
    cache_path: impl AsRef<Path>,
) -> Result<(EpistemeValidationReport, EpistemeValidationHashCacheReport), EpistemeError> {
    let episteme_root = episteme_root.as_ref();
    let corpus_root = corpus_root.as_ref();
    let cache_path = cache_path.as_ref();
    let manifest = read_source_manifest(episteme_root)?;
    let paths = source_contract_paths(episteme_root)?;
    let corpus_dir = paths.corpus_dir(episteme_root)?;
    let files_path = corpus_dir.join(&manifest.files);
    let queue_path = corpus_dir.join(&manifest.extraction_queue);
    let files = read_files_tsv(&files_path)?;
    let queue = read_queue_tsv(&queue_path)?;
    let mut errors = Vec::new();
    let mapping_ledger = validate_mapping_ledger(episteme_root, &mut errors)?;
    let mut hash_cache = EpistemeValidationHashCache::load(cache_path);
    errors.extend(validate_contract_with_hash_cache(
        corpus_root,
        &manifest,
        &files,
        &queue,
        Some(&mut hash_cache),
    )?);
    let cache_report = hash_cache.write(cache_path)?;

    Ok((
        EpistemeValidationReport {
            schema_version: VALIDATION_SCHEMA_VERSION,
            passed: errors.is_empty(),
            errors,
            files_tsv_rows: files.len(),
            extraction_queue_rows: queue.len(),
            primary_language: manifest.primary_language,
            corpus_root_env: manifest.corpus_root_env,
            raw_to_rdf_promotion_allowed: manifest.raw_to_rdf_promotion_allowed,
            mapping_ledger_sections: mapping_ledger.section_count,
            mapping_ledger_reasoning_property_records: mapping_ledger
                .reasoning_property_record_count,
        },
        cache_report,
    ))
}

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

/// Validate a episteme source-contract source contract from Rust.
///
/// # Errors
///
/// Returns an error when the manifest or TSV files cannot be read or parsed.
pub fn validate_episteme_source_contract(
    episteme_root: impl AsRef<Path>,
    corpus_root: impl AsRef<Path>,
) -> Result<EpistemeValidationReport, EpistemeError> {
    let episteme_root = episteme_root.as_ref();
    let corpus_root = corpus_root.as_ref();
    let manifest = read_source_manifest(episteme_root)?;
    let paths = source_contract_paths(episteme_root)?;
    let corpus_dir = paths.corpus_dir(episteme_root)?;
    let files_path = corpus_dir.join(&manifest.files);
    let queue_path = corpus_dir.join(&manifest.extraction_queue);
    let files = read_files_tsv(&files_path)?;
    let queue = read_queue_tsv(&queue_path)?;
    let mut errors = Vec::new();
    let mapping_ledger = validate_mapping_ledger(episteme_root, &mut errors)?;
    errors.extend(validate_contract_with_hash_cache(
        corpus_root,
        &manifest,
        &files,
        &queue,
        None,
    )?);

    Ok(EpistemeValidationReport {
        schema_version: VALIDATION_SCHEMA_VERSION,
        passed: errors.is_empty(),
        errors,
        files_tsv_rows: files.len(),
        extraction_queue_rows: queue.len(),
        primary_language: manifest.primary_language,
        corpus_root_env: manifest.corpus_root_env,
        raw_to_rdf_promotion_allowed: manifest.raw_to_rdf_promotion_allowed,
        mapping_ledger_sections: mapping_ledger.section_count,
        mapping_ledger_reasoning_property_records: mapping_ledger.reasoning_property_record_count,
    })
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

fn read_source_manifest(episteme_root: &Path) -> Result<EpistemeSourceManifest, EpistemeError> {
    let paths = source_contract_paths(episteme_root)?;
    let path = paths.source_manifest_path(episteme_root);
    let raw = read_to_string(&path)?;
    let manifest = parse_episteme_source_manifest_toml(&raw)
        .map_err(|source| EpistemeError::Parse { path, source })?;
    if manifest.domain != paths.domain_id() {
        return Err(EpistemeError::InvalidEpistemeManifest(format!(
            "selected source manifest domain `{}` does not match selected manifest domain `{}`",
            manifest.domain,
            paths.domain_id()
        )));
    }
    Ok(manifest)
}

pub(super) fn read_mapping_ledger_raw(episteme_root: &Path) -> Result<String, EpistemeError> {
    let paths = source_contract_paths(episteme_root)?;
    read_to_string(&paths.mapping_ledger_path(episteme_root))
}

pub(super) fn source_contract_paths(
    episteme_root: &Path,
) -> Result<EpistemeSourceContractPaths, EpistemeError> {
    let manifest_path = episteme_root.join(ONTOLOGY_MANIFEST_RELATIVE_PATH);
    if !manifest_path.is_file() {
        return Err(EpistemeError::InvalidEpistemeManifest(format!(
            "missing episteme config: {ONTOLOGY_MANIFEST_RELATIVE_PATH}"
        )));
    }

    let raw = read_to_string(&manifest_path)?;
    let manifest = toml::from_str::<EpistemeOntologyManifest>(&raw).map_err(|source| {
        EpistemeError::EpistemeManifestToml {
            path: manifest_path.clone(),
            source,
        }
    })?;
    let selected = select_source_contract_paths(&manifest)?;

    Ok(EpistemeSourceContractPaths {
        domain_id: selected.domain_id,
        source_manifest_relative_path: format!("ontology/{}", selected.source_manifest),
        mapping_ledger_relative_path: format!("ontology/{}", selected.mapping_ledger),
    })
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct SelectedSourceContractPaths {
    domain_id: String,
    source_manifest: String,
    mapping_ledger: String,
}

fn select_source_contract_paths(
    manifest: &EpistemeOntologyManifest,
) -> Result<SelectedSourceContractPaths, EpistemeError> {
    if let Some(active) = &manifest.active_source_contract {
        return select_active_source_contract(manifest, active);
    }

    let declared = declared_source_contracts(manifest)?;
    match declared.as_slice() {
        [selected] => Ok(selected.clone()),
        [] => Err(EpistemeError::InvalidEpistemeManifest(
            "expected at least one declared source manifest and mapping ledger".to_string(),
        )),
        many => Err(EpistemeError::InvalidEpistemeManifest(format!(
            "found {} selectable source contracts; add [active_source_contract] to ontology/manifest.toml",
            many.len()
        ))),
    }
}

fn select_active_source_contract(
    manifest: &EpistemeOntologyManifest,
    active: &EpistemeActiveSourceContract,
) -> Result<SelectedSourceContractPaths, EpistemeError> {
    let source_manifest = normalize_episteme_source_contract_manifest_path(
        &active.source_manifest,
        "source_manifest",
    )?;
    let mapping_ledger =
        normalize_episteme_source_contract_manifest_path(&active.mapping_ledger, "mapping_ledger")?;
    let domain = manifest
        .domains
        .iter()
        .find(|domain| domain.id == active.domain_id)
        .ok_or_else(|| {
            EpistemeError::InvalidEpistemeManifest(format!(
                "active source contract references unknown domain_id: {}",
                active.domain_id
            ))
        })?;
    let declared_source_manifests = normalized_domain_paths(domain, "source_manifests")?;
    if !declared_source_manifests.contains(&source_manifest) {
        return Err(EpistemeError::InvalidEpistemeManifest(format!(
            "active source_manifest is not declared by domain {}: {}",
            active.domain_id, source_manifest
        )));
    }
    let declared_mapping_ledgers = normalized_domain_paths(domain, "mapping_ledgers")?;
    if !declared_mapping_ledgers.contains(&mapping_ledger) {
        return Err(EpistemeError::InvalidEpistemeManifest(format!(
            "active mapping_ledger is not declared by domain {}: {}",
            active.domain_id, mapping_ledger
        )));
    }

    Ok(SelectedSourceContractPaths {
        domain_id: active.domain_id.clone(),
        source_manifest,
        mapping_ledger,
    })
}

fn declared_source_contracts(
    manifest: &EpistemeOntologyManifest,
) -> Result<Vec<SelectedSourceContractPaths>, EpistemeError> {
    let mut selected = Vec::new();
    for domain in &manifest.domains {
        let source_manifests = normalized_domain_paths(domain, "source_manifests")?;
        let mapping_ledgers = normalized_domain_paths(domain, "mapping_ledgers")?;
        if source_manifests.len() == 1 && mapping_ledgers.len() == 1 {
            selected.push(SelectedSourceContractPaths {
                domain_id: domain.id.clone(),
                source_manifest: source_manifests[0].clone(),
                mapping_ledger: mapping_ledgers[0].clone(),
            });
        } else if !source_manifests.is_empty() || !mapping_ledgers.is_empty() {
            return Err(EpistemeError::InvalidEpistemeManifest(format!(
                "domain {} declares {} source manifests and {} mapping ledgers; add [active_source_contract]",
                domain.id,
                source_manifests.len(),
                mapping_ledgers.len()
            )));
        }
    }
    Ok(selected)
}

fn normalized_domain_paths(
    domain: &EpistemeDomainManifest,
    field: &str,
) -> Result<Vec<String>, EpistemeError> {
    let values = match field {
        "source_manifests" => &domain.source_manifests,
        "mapping_ledgers" => &domain.mapping_ledgers,
        _ => unreachable!("unsupported episteme domain path field"),
    };
    values
        .iter()
        .map(|path| normalize_episteme_source_contract_manifest_path(path, field))
        .collect()
}

fn normalize_episteme_source_contract_manifest_path(
    raw: &str,
    field: &str,
) -> Result<String, EpistemeError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(EpistemeError::InvalidEpistemeManifest(format!(
            "`{field}` entries must not be blank"
        )));
    }
    let path = Path::new(trimmed);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(EpistemeError::InvalidEpistemeManifest(format!(
            "`{field}` entries must be safe paths relative to ontology/: {trimmed}"
        )));
    }
    Ok(trimmed.replace('\\', "/"))
}

fn read_to_string(path: &Path) -> Result<String, EpistemeError> {
    fs::read_to_string(path).map_err(|source| EpistemeError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn read_files_tsv(path: &Path) -> Result<Vec<EpistemeFileRow>, EpistemeError> {
    let raw = read_to_string(path)?;
    parse_episteme_files_tsv(&raw).map_err(|source| EpistemeError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

fn read_queue_tsv(path: &Path) -> Result<Vec<EpistemeExtractionQueueRow>, EpistemeError> {
    let raw = read_to_string(path)?;
    parse_episteme_extraction_queue_tsv(&raw).map_err(|source| EpistemeError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

fn validate_mapping_ledger(
    episteme_root: &Path,
    errors: &mut Vec<String>,
) -> Result<EpistemeMappingLedgerValidation, EpistemeError> {
    let paths = source_contract_paths(episteme_root)?;
    let path = paths.mapping_ledger_path(episteme_root);
    let raw = read_to_string(&path)?;
    match validate_episteme_mapping_ledger_org(&raw, paths.mapping_ledger_relative_path()) {
        Ok(validation) => Ok(validation),
        Err(error) => {
            errors.push(format!(
                "mapping ledger `{}` is invalid: {error}",
                path.display()
            ));
            Ok(EpistemeMappingLedgerValidation {
                section_count: 0,
                reasoning_property_record_count: 0,
            })
        }
    }
}

fn validate_contract_with_hash_cache(
    corpus_root: &Path,
    manifest: &EpistemeSourceManifest,
    files: &[EpistemeFileRow],
    queue: &[EpistemeExtractionQueueRow],
    mut hash_cache: Option<&mut EpistemeValidationHashCache>,
) -> Result<Vec<String>, EpistemeError> {
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
        return Ok(errors);
    }

    let extension_routes = extension_routes(manifest);
    let discovered_paths = discovered_corpus_paths(corpus_root, &manifest.ignored_names)?;
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
        let source_path = corpus_root.join(&row.relative_path);
        if !source_path.is_file() {
            errors.push(format!(
                "row {row_number} missing source file: {}",
                row.relative_path
            ));
            continue;
        }
        let metadata = fs::metadata(&source_path).map_err(|source| EpistemeError::Io {
            path: source_path.clone(),
            source,
        })?;
        if metadata.len() != row.byte_size {
            errors.push(format!(
                "row {row_number} byte_size drift: {}",
                row.relative_path
            ));
        }
        let actual_hash = if let Some(cache) = hash_cache.as_deref_mut() {
            cache.sha256_for(row, &source_path, &metadata)?
        } else {
            sha256_file(&source_path)?
        };
        if actual_hash != row.sha256 {
            errors.push(format!(
                "row {row_number} sha256 drift: {}",
                row.relative_path
            ));
        }
    }

    for path in discovered_paths.difference(&file_paths) {
        errors.push(format!("files.tsv missing corpus file: {path}"));
    }
    for path in file_paths.difference(&discovered_paths) {
        errors.push(format!("files.tsv contains non-corpus file: {path}"));
    }

    validate_queue_rows(queue, files, &mut errors);
    Ok(errors)
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
struct EpistemeValidationHashCacheFile {
    schema_version: String,
    entries: BTreeMap<String, EpistemeValidationHashCacheEntry>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
struct EpistemeValidationHashCacheEntry {
    relative_path: String,
    byte_size: u64,
    modified_unix_seconds: u64,
    modified_nanos: u32,
    sha256: String,
}

#[derive(Debug, Clone)]
struct EpistemeValidationHashCache {
    entries: BTreeMap<String, EpistemeValidationHashCacheEntry>,
    next_entries: BTreeMap<String, EpistemeValidationHashCacheEntry>,
    entries_loaded: usize,
    malformed_cache: bool,
    hash_cache_hits: usize,
    hash_cache_misses: usize,
    stale_entries: usize,
}

impl EpistemeValidationHashCache {
    fn load(path: &Path) -> Self {
        let loaded = fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str::<EpistemeValidationHashCacheFile>(&raw).ok());
        match loaded {
            Some(file) => {
                if file.schema_version == VALIDATION_HASH_CACHE_SCHEMA_VERSION {
                    Self {
                        entries_loaded: file.entries.len(),
                        entries: file.entries,
                        next_entries: BTreeMap::new(),
                        malformed_cache: false,
                        hash_cache_hits: 0,
                        hash_cache_misses: 0,
                        stale_entries: 0,
                    }
                } else {
                    Self::malformed()
                }
            }
            None if path.exists() => Self::malformed(),
            None => Self {
                entries: BTreeMap::new(),
                next_entries: BTreeMap::new(),
                entries_loaded: 0,
                malformed_cache: false,
                hash_cache_hits: 0,
                hash_cache_misses: 0,
                stale_entries: 0,
            },
        }
    }

    fn malformed() -> Self {
        Self {
            entries: BTreeMap::new(),
            next_entries: BTreeMap::new(),
            entries_loaded: 0,
            malformed_cache: true,
            hash_cache_hits: 0,
            hash_cache_misses: 0,
            stale_entries: 0,
        }
    }

    fn sha256_for(
        &mut self,
        row: &EpistemeFileRow,
        source_path: &Path,
        metadata: &fs::Metadata,
    ) -> Result<String, EpistemeError> {
        let fingerprint = file_fingerprint(metadata);
        if let Some(entry) = self.entries.get(row.relative_path.as_str()) {
            if entry.byte_size == metadata.len()
                && entry.modified_unix_seconds == fingerprint.modified_unix_seconds
                && entry.modified_nanos == fingerprint.modified_nanos
                && entry.sha256 == row.sha256
            {
                self.hash_cache_hits += 1;
                self.next_entries
                    .insert(row.relative_path.clone(), entry.clone());
                return Ok(entry.sha256.clone());
            }
            self.stale_entries += 1;
        }

        self.hash_cache_misses += 1;
        let actual_hash = sha256_file(source_path)?;
        if actual_hash == row.sha256 && metadata.len() == row.byte_size {
            self.next_entries.insert(
                row.relative_path.clone(),
                EpistemeValidationHashCacheEntry {
                    relative_path: row.relative_path.clone(),
                    byte_size: metadata.len(),
                    modified_unix_seconds: fingerprint.modified_unix_seconds,
                    modified_nanos: fingerprint.modified_nanos,
                    sha256: actual_hash.clone(),
                },
            );
        }
        Ok(actual_hash)
    }

    fn write(self, path: &Path) -> Result<EpistemeValidationHashCacheReport, EpistemeError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| EpistemeError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let entries_written = self.next_entries.len();
        let file = EpistemeValidationHashCacheFile {
            schema_version: VALIDATION_HASH_CACHE_SCHEMA_VERSION.to_string(),
            entries: self.next_entries,
        };
        let raw = serde_json::to_string_pretty(&file).map_err(|source| EpistemeError::Json {
            path: path.to_path_buf(),
            source,
        })?;
        fs::write(path, format!("{raw}\n")).map_err(|source| EpistemeError::Io {
            path: path.to_path_buf(),
            source,
        })?;

        Ok(EpistemeValidationHashCacheReport {
            schema_version: VALIDATION_HASH_CACHE_REPORT_SCHEMA_VERSION,
            cache_path: path.to_path_buf(),
            entries_loaded: self.entries_loaded,
            malformed_cache: self.malformed_cache,
            hash_cache_hits: self.hash_cache_hits,
            hash_cache_misses: self.hash_cache_misses,
            stale_entries: self.stale_entries,
            entries_written,
        })
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct EpistemeFileFingerprint {
    modified_unix_seconds: u64,
    modified_nanos: u32,
}

fn file_fingerprint(metadata: &fs::Metadata) -> EpistemeFileFingerprint {
    let modified = metadata
        .modified()
        .unwrap_or(SystemTime::UNIX_EPOCH)
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    EpistemeFileFingerprint {
        modified_unix_seconds: modified.as_secs(),
        modified_nanos: modified.subsec_nanos(),
    }
}

fn validate_queue_rows(
    queue: &[EpistemeExtractionQueueRow],
    files: &[EpistemeFileRow],
    errors: &mut Vec<String>,
) {
    let files_by_id = files
        .iter()
        .map(|row| (row.file_id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let mut seen_queue_ids = BTreeSet::new();
    let mut seen_file_ids = BTreeSet::new();
    for (index, row) in queue.iter().enumerate() {
        let row_number = index + 2;
        if !seen_queue_ids.insert(row.queue_id.as_str()) {
            errors.push(format!(
                "duplicate queue_id at row {row_number}: {}",
                row.queue_id
            ));
        }
        if !seen_file_ids.insert(row.file_id.as_str()) {
            errors.push(format!(
                "duplicate queued file_id at row {row_number}: {}",
                row.file_id
            ));
        }
        if row.output_contract != OUTPUT_CONTRACT {
            errors.push(format!(
                "row {row_number} output_contract must be {OUTPUT_CONTRACT}"
            ));
        }
        if row.status != PENDING_STATUS {
            errors.push(format!("row {row_number} status must be {PENDING_STATUS}"));
        }
        let Some(source) = files_by_id.get(row.file_id.as_str()) else {
            errors.push(format!(
                "row {row_number} references unknown file_id: {}",
                row.file_id
            ));
            continue;
        };
        if row.relative_path != source.relative_path
            || row.category != source.category
            || row.language != source.language
            || row.extraction_route != source.extraction_route
        {
            errors.push(format!(
                "row {row_number} does not match files.tsv for {}",
                row.file_id
            ));
        }
    }
    for file in files {
        if !seen_file_ids.contains(file.file_id.as_str()) {
            errors.push(format!(
                "extraction_queue.tsv missing file_id: {}",
                file.file_id
            ));
        }
    }
    if queue.len() != files.len() {
        errors.push(format!(
            "extraction_queue.tsv row count mismatch: {} != {}",
            queue.len(),
            files.len()
        ));
    }
}

fn extension_routes(manifest: &EpistemeSourceManifest) -> BTreeMap<&str, &str> {
    manifest
        .routes
        .iter()
        .flat_map(|(route, extensions)| {
            extensions
                .iter()
                .map(move |extension| (extension.as_str(), route.as_str()))
        })
        .collect()
}

fn discovered_corpus_paths(
    corpus_root: &Path,
    ignored_names: &[String],
) -> Result<BTreeSet<String>, EpistemeError> {
    let ignored = ignored_names
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    WalkDir::new(corpus_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            let relative = entry.path().strip_prefix(corpus_root).ok()?;
            let ignored_path = relative.components().any(|component| {
                let text = component.as_os_str().to_string_lossy();
                ignored.contains(text.as_ref()) || text.starts_with('.')
            });
            (!ignored_path).then(|| relative.to_string_lossy().replace('\\', "/"))
        })
        .collect::<BTreeSet<_>>()
        .pipe(Ok)
}

fn sha256_file(path: &Path) -> Result<String, EpistemeError> {
    let mut file = fs::File::open(path).map_err(|source| EpistemeError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let bytes = file.read(&mut buffer).map_err(|source| EpistemeError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if bytes == 0 {
            break;
        }
        hasher.update(&buffer[..bytes]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn safe_run_id(value: &str) -> Result<(), EpistemeError> {
    let safe = !value.is_empty()
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'));
    if safe {
        Ok(())
    } else {
        Err(EpistemeError::InvalidRunId(value.to_string()))
    }
}

fn count_by<'a>(values: impl Iterator<Item = &'a str>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(value.to_string()).or_insert(0) += 1;
    }
    counts
}

trait Pipe: Sized {
    fn pipe<T>(self, f: impl FnOnce(Self) -> T) -> T {
        f(self)
    }
}

impl<T> Pipe for T {}
