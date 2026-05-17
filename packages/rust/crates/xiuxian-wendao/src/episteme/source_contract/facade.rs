//! Episteme source-contract planning, validation, and read-model facades.
//!
//! This branch keeps raw corpus rows as evidence, validates them through
//! parser-owned contracts, and exposes Rust-owned run-plan, registry, and
//! read-model surfaces for downstream `WendaoGraph` quality checks.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};

use serde::Deserialize;
use thiserror::Error;
use walkdir::WalkDir;
use xiuxian_wendao_parsers::{
    EpistemeExtractionQueueRow, EpistemeFileRow, EpistemeMappingLedgerValidation,
    EpistemeSourceContractParseError, EpistemeSourceManifest, parse_episteme_extraction_queue_tsv,
    parse_episteme_files_tsv, parse_episteme_source_manifest_toml,
    validate_episteme_mapping_ledger_org,
};

#[path = "config.rs"]
mod config;
#[path = "evidence.rs"]
mod evidence;
#[path = "promotion.rs"]
mod promotion;
#[path = "read_model/mod.rs"]
mod read_model;
#[path = "registry.rs"]
mod registry;
#[path = "run_plan.rs"]
mod run_plan;
#[path = "selection.rs"]
mod selection;
#[path = "structure.rs"]
mod structure;
#[path = "validation.rs"]
mod validation;
#[path = "write.rs"]
mod write;

pub use config::{EpistemeRuntimeConfig, load_episteme_runtime_config};
pub use evidence::{
    EpistemeEvidenceByteSizeStatus, EpistemeEvidenceReadReport, EpistemeEvidenceReadRequest,
    EpistemeEvidenceReadValidationMode, EpistemeEvidenceSha256Status,
    EpistemeEvidenceSourceAvailability, EpistemeEvidenceSourceRef, EpistemeEvidenceTextPreview,
    read_episteme_evidence,
};
pub use promotion::{
    EpistemeAudioClaimPromotionProposalReport, EpistemeAudioClaimPromotionProposalRequest,
    write_episteme_audio_claim_promotion_proposal,
};
#[cfg(feature = "julia")]
pub use read_model::build_episteme_wendaograph_quality_request_batches;
pub use read_model::{
    EpistemeAudioEvidenceReadModelRequest, EpistemeAudioEvidenceSegmentRow,
    EpistemeAudioEvidenceSourceRow, EpistemeAudioReviewedClaimObjectKind,
    EpistemeAudioReviewedClaimReadModelRequest, EpistemeAudioReviewedClaimRow,
    EpistemeReadModelMaterialization, EpistemeReadModelRequest, EpistemeReadModelTable,
    materialize_episteme_audio_evidence_review_seed,
    materialize_episteme_audio_reviewed_claim_seed, materialize_episteme_read_model_seed,
    materialize_episteme_read_model_seed_with_validation_hash_cache,
    materialize_episteme_registry_reference_graph_read_model_seed,
    validate_episteme_read_model_relation_endpoints,
};
pub use registry::{
    EpistemeRegistryDomainId, EpistemeRegistryDuplicateDomainId, EpistemeRegistryEntry,
    EpistemeRegistryError, EpistemeRegistryGitMaterializationError, EpistemeRegistryId,
    EpistemeRegistryInvalidDomainId, EpistemeRegistryLoadReceipt,
    EpistemeRegistryMissingExtensionTarget, EpistemeRegistryReferenceGraphEntry,
    EpistemeRegistryReferenceGraphLink, EpistemeRegistryReferenceGraphReceipt,
    LoadedEpistemeRegistryEntry, LoadedEpistemeSourceKind, load_episteme_registry_entries,
    load_episteme_registry_entries_with_mode, validate_episteme_registry_reference_graph,
};
pub use run_plan::{
    EpistemeRunPlanReceipt, EpistemeRunPlanRequest, EpistemeRunTask, plan_episteme_extraction_run,
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
pub use validation::{
    EpistemeValidationHashCacheReport, EpistemeValidationReport, validate_episteme_source_contract,
    validate_episteme_source_contract_with_hash_cache,
};
pub use write::{EpistemeRunPlanWriteReport, write_episteme_extraction_run_plan};

pub(super) const ONTOLOGY_MANIFEST_RELATIVE_PATH: &str = "ontology/manifest.toml";
pub(super) const FILES_TSV: &str = "files.tsv";
pub(super) const EXTRACTION_QUEUE_TSV: &str = "extraction_queue.tsv";
pub(super) const OUTPUT_CONTRACT: &str = "cache_only_no_rdf_promotion";
pub(super) const PENDING_STATUS: &str = "pending";
pub(super) const PLANNED_STATUS: &str = "planned";
pub(super) const RUN_SCHEMA_VERSION: &str = "xiuxian_wendao.episteme_extraction_run_plan.v1";
pub(super) const RUN_PLAN_VALIDATION_MODE_CONTRACT_SHAPE_ONLY: &str = "contract_shape_only";
pub(super) const VALIDATION_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_source_contract_validation.v1";
pub(super) const VALIDATION_HASH_CACHE_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_source_contract_validation_hash_cache.v1";
pub(super) const VALIDATION_HASH_CACHE_REPORT_SCHEMA_VERSION: &str =
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

pub(super) fn read_source_manifest(
    episteme_root: &Path,
) -> Result<EpistemeSourceManifest, EpistemeError> {
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

pub(super) fn read_files_tsv(path: &Path) -> Result<Vec<EpistemeFileRow>, EpistemeError> {
    let raw = read_to_string(path)?;
    parse_episteme_files_tsv(&raw).map_err(|source| EpistemeError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

pub(super) fn read_queue_tsv(
    path: &Path,
) -> Result<Vec<EpistemeExtractionQueueRow>, EpistemeError> {
    let raw = read_to_string(path)?;
    parse_episteme_extraction_queue_tsv(&raw).map_err(|source| EpistemeError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

pub(super) fn validate_mapping_ledger(
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

pub(super) fn validate_queue_rows(
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

pub(super) fn extension_routes(manifest: &EpistemeSourceManifest) -> BTreeMap<&str, &str> {
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

pub(super) fn discovered_corpus_paths(
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

pub(super) fn safe_run_id(value: &str) -> Result<(), EpistemeError> {
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

pub(super) fn count_by<'a>(values: impl Iterator<Item = &'a str>) -> BTreeMap<String, usize> {
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
