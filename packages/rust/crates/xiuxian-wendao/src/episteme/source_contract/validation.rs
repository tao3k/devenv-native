//! Source-contract validation and hash-cache support.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use xiuxian_wendao_parsers::{EpistemeExtractionQueueRow, EpistemeFileRow, EpistemeSourceManifest};

use super::{
    EXTRACTION_QUEUE_TSV, EpistemeError, FILES_TSV, VALIDATION_HASH_CACHE_REPORT_SCHEMA_VERSION,
    VALIDATION_HASH_CACHE_SCHEMA_VERSION, VALIDATION_SCHEMA_VERSION, discovered_corpus_paths,
    extension_routes, read_files_tsv, read_queue_tsv, read_source_manifest, source_contract_paths,
    validate_mapping_ledger, validate_queue_rows,
};

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
    let inputs = validation_inputs(episteme_root.as_ref())?;
    let corpus_root = corpus_root.as_ref();
    let cache_path = cache_path.as_ref();
    let mut errors = Vec::new();
    let mapping_ledger = validate_mapping_ledger(inputs.episteme_root.as_path(), &mut errors)?;
    let mut hash_cache = EpistemeValidationHashCache::load(cache_path);
    errors.extend(validate_contract_with_hash_cache(
        corpus_root,
        &inputs.manifest,
        &inputs.files,
        &inputs.queue,
        Some(&mut hash_cache),
    )?);
    let cache_report = hash_cache.write(cache_path)?;

    Ok((
        EpistemeValidationReport {
            schema_version: VALIDATION_SCHEMA_VERSION,
            passed: errors.is_empty(),
            errors,
            files_tsv_rows: inputs.files.len(),
            extraction_queue_rows: inputs.queue.len(),
            primary_language: inputs.manifest.primary_language,
            corpus_root_env: inputs.manifest.corpus_root_env,
            raw_to_rdf_promotion_allowed: inputs.manifest.raw_to_rdf_promotion_allowed,
            mapping_ledger_sections: mapping_ledger.section_count,
            mapping_ledger_reasoning_property_records: mapping_ledger
                .reasoning_property_record_count,
        },
        cache_report,
    ))
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
    let inputs = validation_inputs(episteme_root.as_ref())?;
    let corpus_root = corpus_root.as_ref();
    let mut errors = Vec::new();
    let mapping_ledger = validate_mapping_ledger(inputs.episteme_root.as_path(), &mut errors)?;
    errors.extend(validate_contract_with_hash_cache(
        corpus_root,
        &inputs.manifest,
        &inputs.files,
        &inputs.queue,
        None,
    )?);

    Ok(EpistemeValidationReport {
        schema_version: VALIDATION_SCHEMA_VERSION,
        passed: errors.is_empty(),
        errors,
        files_tsv_rows: inputs.files.len(),
        extraction_queue_rows: inputs.queue.len(),
        primary_language: inputs.manifest.primary_language,
        corpus_root_env: inputs.manifest.corpus_root_env,
        raw_to_rdf_promotion_allowed: inputs.manifest.raw_to_rdf_promotion_allowed,
        mapping_ledger_sections: mapping_ledger.section_count,
        mapping_ledger_reasoning_property_records: mapping_ledger.reasoning_property_record_count,
    })
}

struct EpistemeValidationInputs {
    episteme_root: PathBuf,
    manifest: EpistemeSourceManifest,
    files: Vec<EpistemeFileRow>,
    queue: Vec<EpistemeExtractionQueueRow>,
}

fn validation_inputs(episteme_root: &Path) -> Result<EpistemeValidationInputs, EpistemeError> {
    let manifest = read_source_manifest(episteme_root)?;
    let paths = source_contract_paths(episteme_root)?;
    let corpus_dir = paths.corpus_dir(episteme_root)?;
    let files = read_files_tsv(&corpus_dir.join(&manifest.files))?;
    let queue = read_queue_tsv(&corpus_dir.join(&manifest.extraction_queue))?;
    Ok(EpistemeValidationInputs {
        episteme_root: episteme_root.to_path_buf(),
        manifest,
        files,
        queue,
    })
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
