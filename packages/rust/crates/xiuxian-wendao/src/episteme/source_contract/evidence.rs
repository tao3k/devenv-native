//! Targeted episteme source evidence reads.

use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use serde::Serialize;
use xiuxian_wendao_parsers::EpistemeSourceManifest;

use super::{
    EpistemeError, EpistemeFileRow, read_files_tsv, read_source_manifest, source_contract_paths,
    structure::validate_source_contract_metadata_only, validate_episteme_source_contract,
};

const EVIDENCE_READ_REPORT_SCHEMA_VERSION: &str = "xiuxian_wendao.episteme_evidence_read_report.v1";
const DEFAULT_MAX_PREVIEW_BYTES: usize = 8192;
const TEXT_PREVIEW_KIND: &str = "plain-text";
const UNSUPPORTED_BINARY_PREVIEW_KIND: &str = "unsupported-binary";

/// Validation policy used while reading one targeted episteme evidence source.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EpistemeEvidenceReadValidationMode {
    /// Validate manifest and file metadata without hashing file contents.
    #[default]
    MetadataOnly,
    /// Run the full source-contract validation, including sha256 drift checks.
    FullHash,
}

/// Request for reading a single source-contract evidence row by file id.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeEvidenceReadRequest {
    /// Episteme repository root.
    pub episteme_root: PathBuf,
    /// Raw corpus root.
    pub corpus_root: PathBuf,
    /// Source-contract file id to resolve.
    pub file_id: String,
    /// Maximum bytes to include in a text preview.
    pub max_preview_bytes: usize,
    /// Validation policy for this evidence read.
    pub validation_mode: EpistemeEvidenceReadValidationMode,
}

impl EpistemeEvidenceReadRequest {
    /// Create a request for targeted Rust-owned evidence reading.
    #[must_use]
    pub fn new(
        episteme_root: impl Into<PathBuf>,
        corpus_root: impl Into<PathBuf>,
        file_id: impl Into<String>,
    ) -> Self {
        Self {
            episteme_root: episteme_root.into(),
            corpus_root: corpus_root.into(),
            file_id: file_id.into(),
            max_preview_bytes: DEFAULT_MAX_PREVIEW_BYTES,
            validation_mode: EpistemeEvidenceReadValidationMode::default(),
        }
    }

    /// Set the maximum text preview bytes.
    #[must_use]
    pub fn with_max_preview_bytes(mut self, max_preview_bytes: usize) -> Self {
        self.max_preview_bytes = max_preview_bytes;
        self
    }

    /// Set the validation policy.
    #[must_use]
    pub fn with_validation_mode(
        mut self,
        validation_mode: EpistemeEvidenceReadValidationMode,
    ) -> Self {
        self.validation_mode = validation_mode;
        self
    }
}

/// Source row metadata resolved for a targeted evidence read.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeEvidenceSourceRef {
    /// Source-contract file id.
    pub file_id: String,
    /// Source-relative path from `files.tsv`.
    pub relative_path: String,
    /// File extension from `files.tsv`.
    pub extension: String,
    /// Expected byte size from `files.tsv`.
    pub byte_size: u64,
    /// Expected sha256 from `files.tsv`.
    pub sha256: String,
    /// Source category from `files.tsv`.
    pub category: String,
    /// Source language from `files.tsv`.
    pub language: String,
    /// Intended extraction route from `files.tsv`.
    pub extraction_route: String,
    /// Resolved source path under the caller-provided corpus root.
    pub source_path: PathBuf,
}

/// Bounded text preview for a plain-text evidence source.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeEvidenceTextPreview {
    /// Preview text.
    pub text: String,
    /// Number of bytes returned after UTF-8 conversion.
    pub byte_count: usize,
    /// True when the source exceeded the requested preview window.
    pub truncated: bool,
}

/// Availability of the resolved evidence source.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EpistemeEvidenceSourceAvailability {
    /// The source path was resolved from the source contract and exists.
    Available,
}

/// Byte-size status for the resolved evidence source.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EpistemeEvidenceByteSizeStatus {
    /// Filesystem byte size matches the source contract row.
    Matches,
    /// Filesystem byte size differs from the source contract row.
    Drift,
}

/// Sha256 proof status for the resolved evidence source.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EpistemeEvidenceSha256Status {
    /// Full-hash validation was requested and passed.
    Verified,
    /// This read used metadata-only validation, so sha256 was not recomputed.
    NotChecked,
}

/// Report emitted after reading one targeted source evidence row.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeEvidenceReadReport {
    /// Report schema version.
    pub schema_version: &'static str,
    /// Selected source-contract domain.
    pub domain: String,
    /// Primary source-contract language.
    pub primary_language: String,
    /// Resolved source row metadata.
    pub source: EpistemeEvidenceSourceRef,
    /// Availability of the resolved source path.
    pub source_availability: EpistemeEvidenceSourceAvailability,
    /// Byte-size comparison status.
    pub byte_size_status: EpistemeEvidenceByteSizeStatus,
    /// Sha256 proof status.
    pub sha256_status: EpistemeEvidenceSha256Status,
    /// Validation policy used for this evidence read.
    pub validation_mode: EpistemeEvidenceReadValidationMode,
    /// Preview kind for the source.
    pub preview_kind: String,
    /// Bounded plain-text preview, when supported.
    pub text_preview: Option<EpistemeEvidenceTextPreview>,
    /// Whether extraction ran during this read.
    pub extraction_executed: bool,
    /// Whether raw rows may be promoted directly to RDF truth.
    pub raw_to_rdf_promotion_allowed: bool,
}

/// Read one source-contract evidence row by `file_id`.
///
/// # Errors
///
/// Returns an error when source-contract validation fails, the file id is
/// unknown, the source file cannot be accessed, or a text preview source is not
/// valid UTF-8.
pub fn read_episteme_evidence(
    request: &EpistemeEvidenceReadRequest,
) -> Result<EpistemeEvidenceReadReport, EpistemeError> {
    let manifest = read_source_manifest(&request.episteme_root)?;
    let paths = source_contract_paths(&request.episteme_root)?;
    let corpus_dir = paths.corpus_dir(&request.episteme_root)?;
    let files = read_files_tsv(&corpus_dir.join(&manifest.files))?;
    validate_for_mode(request, &manifest, &files)?;

    let Some(row) = files.iter().find(|row| row.file_id == request.file_id) else {
        return Err(EpistemeError::InvalidContract(vec![format!(
            "unknown source-contract file_id: {}",
            request.file_id
        )]));
    };
    let source_path = request.corpus_root.join(&row.relative_path);
    let metadata = fs::metadata(&source_path).map_err(|source| EpistemeError::Io {
        path: source_path.clone(),
        source,
    })?;
    let source = source_ref(row, source_path);
    let text_preview = if is_text_preview_supported(&row.extension) {
        Some(read_text_preview(
            &source.source_path,
            request.max_preview_bytes,
        )?)
    } else {
        None
    };
    let preview_kind = if text_preview.is_some() {
        TEXT_PREVIEW_KIND
    } else {
        UNSUPPORTED_BINARY_PREVIEW_KIND
    };

    Ok(EpistemeEvidenceReadReport {
        schema_version: EVIDENCE_READ_REPORT_SCHEMA_VERSION,
        domain: manifest.domain,
        primary_language: manifest.primary_language,
        source,
        source_availability: EpistemeEvidenceSourceAvailability::Available,
        byte_size_status: byte_size_status(metadata.len(), row.byte_size),
        sha256_status: sha256_status(request.validation_mode),
        validation_mode: request.validation_mode,
        preview_kind: preview_kind.to_string(),
        text_preview,
        extraction_executed: false,
        raw_to_rdf_promotion_allowed: false,
    })
}

fn byte_size_status(actual: u64, expected: u64) -> EpistemeEvidenceByteSizeStatus {
    if actual == expected {
        EpistemeEvidenceByteSizeStatus::Matches
    } else {
        EpistemeEvidenceByteSizeStatus::Drift
    }
}

fn sha256_status(
    validation_mode: EpistemeEvidenceReadValidationMode,
) -> EpistemeEvidenceSha256Status {
    match validation_mode {
        EpistemeEvidenceReadValidationMode::FullHash => EpistemeEvidenceSha256Status::Verified,
        EpistemeEvidenceReadValidationMode::MetadataOnly => {
            EpistemeEvidenceSha256Status::NotChecked
        }
    }
}

fn validate_for_mode(
    request: &EpistemeEvidenceReadRequest,
    manifest: &EpistemeSourceManifest,
    files: &[EpistemeFileRow],
) -> Result<(), EpistemeError> {
    let errors = match request.validation_mode {
        EpistemeEvidenceReadValidationMode::FullHash => {
            let validation =
                validate_episteme_source_contract(&request.episteme_root, &request.corpus_root)?;
            if validation.passed {
                Vec::new()
            } else {
                validation.errors
            }
        }
        EpistemeEvidenceReadValidationMode::MetadataOnly => {
            validate_source_contract_metadata_only(&request.corpus_root, manifest, files)?
        }
    };
    if errors.is_empty() {
        Ok(())
    } else {
        Err(EpistemeError::InvalidContract(errors))
    }
}

fn source_ref(row: &EpistemeFileRow, source_path: PathBuf) -> EpistemeEvidenceSourceRef {
    EpistemeEvidenceSourceRef {
        file_id: row.file_id.clone(),
        relative_path: row.relative_path.clone(),
        extension: row.extension.clone(),
        byte_size: row.byte_size,
        sha256: row.sha256.clone(),
        category: row.category.clone(),
        language: row.language.clone(),
        extraction_route: row.extraction_route.clone(),
        source_path,
    }
}

fn read_text_preview(
    source_path: &Path,
    max_preview_bytes: usize,
) -> Result<EpistemeEvidenceTextPreview, EpistemeError> {
    let read_limit = max_preview_bytes.saturating_add(1);
    let mut file = fs::File::open(source_path).map_err(|source| EpistemeError::Io {
        path: source_path.to_path_buf(),
        source,
    })?;
    let mut buffer = Vec::with_capacity(read_limit);
    file.by_ref()
        .take(read_limit as u64)
        .read_to_end(&mut buffer)
        .map_err(|source| EpistemeError::Io {
            path: source_path.to_path_buf(),
            source,
        })?;
    let truncated = buffer.len() > max_preview_bytes;
    if truncated {
        buffer.truncate(max_preview_bytes);
        while std::str::from_utf8(&buffer).is_err() {
            let Some(_) = buffer.pop() else {
                break;
            };
        }
    }
    let text = String::from_utf8(buffer).map_err(|source| {
        EpistemeError::InvalidContract(vec![format!(
            "text preview source is not valid UTF-8: {}; {source}",
            source_path.display()
        )])
    })?;
    Ok(EpistemeEvidenceTextPreview {
        byte_count: text.len(),
        text,
        truncated,
    })
}

fn is_text_preview_supported(extension: &str) -> bool {
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "csv"
            | "json"
            | "md"
            | "org"
            | "rdf"
            | "sql"
            | "toml"
            | "tsv"
            | "txt"
            | "xml"
            | "yaml"
            | "yml"
    )
}
