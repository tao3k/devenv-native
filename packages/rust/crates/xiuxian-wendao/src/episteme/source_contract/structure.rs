//! Episteme source structure and TOC ledger writing.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use serde::Serialize;
use sha2::{Digest, Sha256};
use xiuxian_wendao_parsers::EpistemeSourceManifest;

use super::{
    EpistemeError, EpistemeFileRow, FILES_TSV, count_by, discovered_corpus_paths, extension_routes,
    read_files_tsv, read_source_manifest, safe_run_id, source_contract_paths,
    validate_episteme_source_contract,
};

const TOC_WRITE_REPORT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_structure_toc_write_report.v1";
const TOC_RECEIPT_SCHEMA_VERSION: &str = "xiuxian_wendao.episteme_structure_toc_receipt.v1";
const TOC_ORG: &str = "toc.org";
const RECEIPT_JSON: &str = "receipt.json";

/// Validation policy used while writing an episteme structure TOC.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EpistemeStructureTocValidationMode {
    /// Validate manifest/file metadata without reading file contents for sha256.
    #[default]
    MetadataOnly,
    /// Run the full source-contract validation, including sha256 drift checks.
    FullHash,
}

/// Request for writing an evidence-only episteme structure TOC Org ledger.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeStructureTocRequest {
    /// Episteme repository root.
    pub episteme_root: PathBuf,
    /// Raw corpus root.
    pub corpus_root: PathBuf,
    /// Safe ASCII run id.
    pub run_id: String,
    /// Validation policy for this structure TOC run.
    pub validation_mode: EpistemeStructureTocValidationMode,
}

impl EpistemeStructureTocRequest {
    /// Create a request for Rust-owned episteme structure TOC writing.
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
            validation_mode: EpistemeStructureTocValidationMode::default(),
        }
    }

    /// Set the validation policy for this structure TOC run.
    #[must_use]
    pub fn with_validation_mode(
        mut self,
        validation_mode: EpistemeStructureTocValidationMode,
    ) -> Self {
        self.validation_mode = validation_mode;
        self
    }
}

/// Receipt persisted beside a generated episteme structure TOC ledger.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeStructureTocReceipt {
    /// Receipt schema version.
    pub schema_version: &'static str,
    /// Safe ASCII run id.
    pub run_id: String,
    /// Selected source-contract domain.
    pub domain: String,
    /// Primary source-contract language.
    pub primary_language: String,
    /// Number of source files represented in the TOC.
    pub file_count: usize,
    /// File counts by extraction route.
    pub route_counts: BTreeMap<String, usize>,
    /// File counts by category.
    pub category_counts: BTreeMap<String, usize>,
    /// Whether raw rows may be promoted directly to RDF truth.
    pub raw_to_rdf_promotion_allowed: bool,
    /// Whether OCR, ASR, or LLM extraction ran during this TOC build.
    pub extraction_executed: bool,
    /// Validation policy used for this structure TOC run.
    pub validation_mode: EpistemeStructureTocValidationMode,
}

/// Report emitted after writing an episteme structure TOC ledger.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeStructureTocWriteReport {
    /// Report schema version.
    pub schema_version: &'static str,
    /// Safe ASCII run id.
    pub run_id: String,
    /// Concrete run directory.
    pub run_dir: PathBuf,
    /// Written Org TOC path.
    pub toc_org_path: PathBuf,
    /// Written receipt JSON path.
    pub receipt_path: PathBuf,
    /// Number of source files represented in the TOC.
    pub file_count: usize,
    /// Whether extraction ran during TOC writing.
    pub extraction_executed: bool,
    /// Whether direct RDF promotion is allowed.
    pub raw_to_rdf_promotion_allowed: bool,
    /// Validation policy used for this structure TOC run.
    pub validation_mode: EpistemeStructureTocValidationMode,
}

/// Write an evidence-only Org TOC ledger for a selected episteme source
/// contract.
///
/// # Errors
///
/// Returns an error when source validation fails, the run id is unsafe, or the
/// TOC/receipt files cannot be written.
pub fn write_episteme_structure_toc(
    request: &EpistemeStructureTocRequest,
    run_root: impl AsRef<Path>,
) -> Result<EpistemeStructureTocWriteReport, EpistemeError> {
    safe_run_id(request.run_id.as_str())?;
    let manifest = read_source_manifest(&request.episteme_root)?;
    let paths = source_contract_paths(&request.episteme_root)?;
    let corpus_dir = paths.corpus_dir(&request.episteme_root)?;
    let files = read_files_tsv(&corpus_dir.join(&manifest.files))?;
    validate_for_mode(request, &manifest, &files)?;
    let receipt = EpistemeStructureTocReceipt {
        schema_version: TOC_RECEIPT_SCHEMA_VERSION,
        run_id: request.run_id.clone(),
        domain: manifest.domain.clone(),
        primary_language: manifest.primary_language.clone(),
        file_count: files.len(),
        route_counts: count_by(files.iter().map(|file| file.extraction_route.as_str())),
        category_counts: count_by(files.iter().map(|file| file.category.as_str())),
        raw_to_rdf_promotion_allowed: false,
        extraction_executed: false,
        validation_mode: request.validation_mode,
    };

    let run_dir = run_root.as_ref().join(&request.run_id);
    let toc_org_path = run_dir.join(TOC_ORG);
    let receipt_path = run_dir.join(RECEIPT_JSON);
    create_dir_all(run_dir.as_path())?;
    write_toc_org(
        toc_org_path.as_path(),
        &receipt,
        &files,
        paths.source_manifest_relative_path(),
    )?;
    write_receipt_json(receipt_path.as_path(), &receipt)?;

    Ok(EpistemeStructureTocWriteReport {
        schema_version: TOC_WRITE_REPORT_SCHEMA_VERSION,
        run_id: request.run_id.clone(),
        run_dir,
        toc_org_path,
        receipt_path,
        file_count: receipt.file_count,
        extraction_executed: false,
        raw_to_rdf_promotion_allowed: false,
        validation_mode: request.validation_mode,
    })
}

fn validate_for_mode(
    request: &EpistemeStructureTocRequest,
    manifest: &EpistemeSourceManifest,
    files: &[EpistemeFileRow],
) -> Result<(), EpistemeError> {
    let errors = match request.validation_mode {
        EpistemeStructureTocValidationMode::FullHash => {
            let validation =
                validate_episteme_source_contract(&request.episteme_root, &request.corpus_root)?;
            if validation.passed {
                Vec::new()
            } else {
                validation.errors
            }
        }
        EpistemeStructureTocValidationMode::MetadataOnly => {
            validate_source_contract_metadata_only(&request.corpus_root, manifest, files)?
        }
    };
    if errors.is_empty() {
        Ok(())
    } else {
        Err(EpistemeError::InvalidContract(errors))
    }
}

pub(super) fn validate_source_contract_metadata_only(
    corpus_root: &Path,
    manifest: &EpistemeSourceManifest,
    files: &[EpistemeFileRow],
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
    }

    for path in discovered_paths.difference(&file_paths) {
        errors.push(format!("files.tsv missing corpus file: {path}"));
    }
    for path in file_paths.difference(&discovered_paths) {
        errors.push(format!("files.tsv contains non-corpus file: {path}"));
    }

    Ok(errors)
}

fn create_dir_all(path: &Path) -> Result<(), EpistemeError> {
    fs::create_dir_all(path).map_err(|source| EpistemeError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn write_toc_org(
    path: &Path,
    receipt: &EpistemeStructureTocReceipt,
    files: &[EpistemeFileRow],
    source_manifest_path: &str,
) -> Result<(), EpistemeError> {
    let mut file = fs::File::create(path).map_err(|source| EpistemeError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    write_toc_header(file.by_ref(), path, receipt)?;
    write_source_manifest_summary(file.by_ref(), path, receipt, source_manifest_path)?;
    write_count_summary(
        file.by_ref(),
        path,
        "Extraction route summary",
        "extraction_route",
        &receipt.route_counts,
    )?;
    write_count_summary(
        file.by_ref(),
        path,
        "Category summary",
        "category",
        &receipt.category_counts,
    )?;
    write_source_files_table(file.by_ref(), path, files)?;
    Ok(())
}

fn write_toc_header(
    mut writer: impl Write,
    path: &Path,
    receipt: &EpistemeStructureTocReceipt,
) -> Result<(), EpistemeError> {
    write_line(
        writer.by_ref(),
        path,
        "#+TITLE: Episteme Source Structure TOC",
    )?;
    write_line(writer.by_ref(), path, "#+OPTIONS: toc:nil")?;
    write_line(writer.by_ref(), path, "")?;
    write_line(writer.by_ref(), path, "* Source structure TOC")?;
    write_line(writer.by_ref(), path, ":PROPERTIES:")?;
    write_line(
        writer.by_ref(),
        path,
        format!(
            ":ID: {}",
            deterministic_uuid("toc", receipt.run_id.as_str())
        )
        .as_str(),
    )?;
    write_line(
        writer.by_ref(),
        path,
        ":WENDAO_KIND: episteme_structure_toc",
    )?;
    write_line(
        writer.by_ref(),
        path,
        ":ONTOLOGY_KIND: source_structure_toc",
    )?;
    write_line(
        writer.by_ref(),
        path,
        format!(":RUN_ID: {}", receipt.run_id).as_str(),
    )?;
    write_line(
        writer.by_ref(),
        path,
        format!(":DOMAIN: {}", receipt.domain).as_str(),
    )?;
    write_line(
        writer.by_ref(),
        path,
        format!(":PRIMARY_LANGUAGE: {}", receipt.primary_language).as_str(),
    )?;
    write_line(writer.by_ref(), path, ":PROMOTION_STATE: evidence_only")?;
    write_line(writer.by_ref(), path, ":LIFECYCLE_STATE: generated")?;
    write_line(writer.by_ref(), path, ":END:")?;
    write_line(writer.by_ref(), path, "")?;
    write_line(
        writer.by_ref(),
        path,
        "This ledger records source structure facts only. It does not embed raw corpus text, execute extraction, or promote RDF truth.",
    )?;
    Ok(())
}

fn write_source_manifest_summary(
    mut writer: impl Write,
    path: &Path,
    receipt: &EpistemeStructureTocReceipt,
    source_manifest_path: &str,
) -> Result<(), EpistemeError> {
    write_line(writer.by_ref(), path, "")?;
    write_line(writer.by_ref(), path, "** Source manifest")?;
    write_line(writer.by_ref(), path, "| key | value |")?;
    write_line(writer.by_ref(), path, "|---+---|")?;
    write_line(
        writer.by_ref(),
        path,
        format!("| source_manifest | {} |", org_cell(source_manifest_path)).as_str(),
    )?;
    write_line(
        writer.by_ref(),
        path,
        format!("| file_count | {} |", receipt.file_count).as_str(),
    )?;
    Ok(())
}

fn write_source_files_table(
    mut writer: impl Write,
    path: &Path,
    files: &[EpistemeFileRow],
) -> Result<(), EpistemeError> {
    write_line(writer.by_ref(), path, "")?;
    write_line(writer.by_ref(), path, "** Source files")?;
    write_line(
        writer.by_ref(),
        path,
        "| file_id | relative_path | extension | byte_size | sha256 | category | language | extraction_route |",
    )?;
    write_line(writer.by_ref(), path, "|---+---+---+---+---+---+---+---|")?;
    for source in files {
        write_line(
            writer.by_ref(),
            path,
            format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} |",
                org_cell(&source.file_id),
                org_cell(&source.relative_path),
                org_cell(&source.extension),
                source.byte_size,
                org_cell(&source.sha256),
                org_cell(&source.category),
                org_cell(&source.language),
                org_cell(&source.extraction_route),
            )
            .as_str(),
        )?;
    }
    Ok(())
}

fn write_count_summary(
    mut writer: impl Write,
    path: &Path,
    title: &str,
    key_label: &str,
    counts: &BTreeMap<String, usize>,
) -> Result<(), EpistemeError> {
    write_line(writer.by_ref(), path, "")?;
    write_line(writer.by_ref(), path, format!("** {title}").as_str())?;
    write_line(
        writer.by_ref(),
        path,
        format!("| {key_label} | file_count |").as_str(),
    )?;
    write_line(writer.by_ref(), path, "|---+---|")?;
    for (key, count) in counts {
        write_line(
            writer.by_ref(),
            path,
            format!("| {} | {count} |", org_cell(key)).as_str(),
        )?;
    }
    Ok(())
}

fn write_receipt_json(
    path: &Path,
    receipt: &EpistemeStructureTocReceipt,
) -> Result<(), EpistemeError> {
    let raw = serde_json::to_string_pretty(receipt).map_err(|source| EpistemeError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    fs::write(path, format!("{raw}\n")).map_err(|source| EpistemeError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn write_line(mut writer: impl Write, path: &Path, line: &str) -> Result<(), EpistemeError> {
    writeln!(writer, "{line}").map_err(|source| EpistemeError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn org_cell(value: &str) -> String {
    value
        .replace('|', "\\vert{}")
        .replace(['\n', '\r', '\t'], " ")
}

fn deterministic_uuid(namespace: &str, value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(namespace.as_bytes());
    hasher.update([0]);
    hasher.update(value.as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15],
    )
}
