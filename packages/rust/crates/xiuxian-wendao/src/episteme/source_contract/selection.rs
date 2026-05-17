//! Episteme evidence selection plan writing.

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
    EpistemeError, EpistemeFileRow, count_by, read_files_tsv, read_source_manifest, safe_run_id,
    source_contract_paths, structure::validate_source_contract_metadata_only,
    validate_episteme_source_contract,
};

const SELECTION_WRITE_REPORT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_evidence_selection_write_report.v1";
const SELECTION_RECEIPT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_evidence_selection_receipt.v1";
const SELECTION_ORG: &str = "selection.org";
pub(super) const SELECTION_TSV: &str = "selection.tsv";
const RECEIPT_JSON: &str = "receipt.json";
const DEFAULT_SELECTION_REASON: &str = "manual_or_agent_selected";
const EVIDENCE_SELECTION_OUTPUT_CONTRACT: &str = "evidence_selection_only_no_rdf_promotion";
const SELECTION_TSV_FIELDS: [&str; 11] = [
    "selection_index",
    "file_id",
    "relative_path",
    "extension",
    "byte_size",
    "sha256",
    "category",
    "language",
    "extraction_route",
    "selection_reason",
    "next_action",
];

/// Validation policy used while writing an episteme evidence selection plan.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EpistemeEvidenceSelectionValidationMode {
    /// Validate manifest and file metadata without hashing file contents.
    #[default]
    MetadataOnly,
    /// Run full source-contract validation, including sha256 drift checks.
    FullHash,
}

/// Request for writing an evidence-only selection plan from chosen file ids.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeEvidenceSelectionPlanRequest {
    /// Episteme repository root.
    pub episteme_root: PathBuf,
    /// Raw corpus root.
    pub corpus_root: PathBuf,
    /// Safe ASCII run id.
    pub run_id: String,
    /// Source-contract file ids selected for downstream evidence work.
    pub file_ids: Vec<String>,
    /// Run-level rationale for this selection.
    pub selection_reason: String,
    /// Validation policy for this selection plan.
    pub validation_mode: EpistemeEvidenceSelectionValidationMode,
}

impl EpistemeEvidenceSelectionPlanRequest {
    /// Create a request for Rust-owned evidence selection writing.
    #[must_use]
    pub fn new(
        episteme_root: impl Into<PathBuf>,
        corpus_root: impl Into<PathBuf>,
        run_id: impl Into<String>,
        file_ids: Vec<String>,
    ) -> Self {
        Self {
            episteme_root: episteme_root.into(),
            corpus_root: corpus_root.into(),
            run_id: run_id.into(),
            file_ids,
            selection_reason: DEFAULT_SELECTION_REASON.to_string(),
            validation_mode: EpistemeEvidenceSelectionValidationMode::default(),
        }
    }

    /// Set a run-level rationale for this selection.
    #[must_use]
    pub fn with_selection_reason(mut self, selection_reason: impl Into<String>) -> Self {
        self.selection_reason = selection_reason.into();
        self
    }

    /// Set the validation policy.
    #[must_use]
    pub fn with_validation_mode(
        mut self,
        validation_mode: EpistemeEvidenceSelectionValidationMode,
    ) -> Self {
        self.validation_mode = validation_mode;
        self
    }
}

/// Raw DTO boundary and stringly state boundary for evidence selection rows.
///
/// One source-contract file selected for downstream evidence work.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeEvidenceSelectionRow {
    /// One-based position in the requested selection order.
    pub selection_index: usize,
    /// Source-contract file id.
    pub file_id: String,
    /// Source path relative to the corpus root.
    pub relative_path: String,
    /// File extension.
    pub extension: String,
    /// Expected byte size from `files.tsv`.
    pub byte_size: u64,
    /// Source SHA-256 copied from `files.tsv`.
    pub source_sha256: String,
    /// Source category.
    pub category: String,
    /// Source language.
    pub language: String,
    /// Intended extraction route from `files.tsv`.
    pub extraction_route: String,
    /// Run-level reason copied into each selected row.
    pub selection_reason: String,
    /// Next extraction route hint. This does not execute extraction.
    pub next_action: String,
}

/// Receipt persisted beside a generated evidence selection plan.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeEvidenceSelectionReceipt {
    /// Receipt schema version.
    pub schema_version: &'static str,
    /// Safe ASCII run id.
    pub run_id: String,
    /// Selected source-contract domain.
    pub domain: String,
    /// Primary source-contract language.
    pub primary_language: String,
    /// Total source file rows available.
    pub source_file_count: usize,
    /// Number of selected rows.
    pub selected_count: usize,
    /// Selected row counts by extraction route.
    pub route_counts: BTreeMap<String, usize>,
    /// Selected row counts by source category.
    pub category_counts: BTreeMap<String, usize>,
    /// Output contract.
    pub output_contract: String,
    /// Whether direct RDF promotion is allowed.
    pub raw_to_rdf_promotion_allowed: bool,
    /// Whether extraction ran during selection writing.
    pub extraction_executed: bool,
    /// Validation policy used for this selection run.
    pub validation_mode: EpistemeEvidenceSelectionValidationMode,
    /// Run-level rationale for this selection.
    pub selection_reason: String,
    /// Selected rows.
    pub selections: Vec<EpistemeEvidenceSelectionRow>,
}

/// Report emitted after writing an evidence selection plan.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeEvidenceSelectionWriteReport {
    /// Report schema version.
    pub schema_version: &'static str,
    /// Safe ASCII run id.
    pub run_id: String,
    /// Concrete run directory.
    pub run_dir: PathBuf,
    /// Written Org ledger path.
    pub selection_org_path: PathBuf,
    /// Written selection TSV path.
    pub selection_tsv_path: PathBuf,
    /// Written receipt JSON path.
    pub receipt_path: PathBuf,
    /// Number of selected rows.
    pub selected_count: usize,
    /// Whether extraction ran during selection writing.
    pub extraction_executed: bool,
    /// Whether direct RDF promotion is allowed.
    pub raw_to_rdf_promotion_allowed: bool,
    /// Validation policy used for this selection run.
    pub validation_mode: EpistemeEvidenceSelectionValidationMode,
}

/// Write a deterministic evidence-only selection plan from source-contract
/// `file_id` values.
///
/// # Errors
///
/// Returns an error when validation fails, the run id is unsafe, the selection
/// is empty, a selected file id is duplicated or unknown, or artifacts cannot
/// be written.
pub fn write_episteme_evidence_selection_plan(
    request: &EpistemeEvidenceSelectionPlanRequest,
    run_root: impl AsRef<Path>,
) -> Result<EpistemeEvidenceSelectionWriteReport, EpistemeError> {
    let receipt = plan_episteme_evidence_selection(request)?;
    let run_dir = run_root.as_ref().join(&receipt.run_id);
    let selection_org_path = run_dir.join(SELECTION_ORG);
    let selection_tsv_path = run_dir.join(SELECTION_TSV);
    let receipt_path = run_dir.join(RECEIPT_JSON);

    create_dir_all(run_dir.as_path())?;
    write_selection_org(selection_org_path.as_path(), &receipt)?;
    write_selection_tsv(selection_tsv_path.as_path(), &receipt.selections)?;
    write_receipt_json(receipt_path.as_path(), &receipt)?;

    Ok(EpistemeEvidenceSelectionWriteReport {
        schema_version: SELECTION_WRITE_REPORT_SCHEMA_VERSION,
        run_id: receipt.run_id,
        run_dir,
        selection_org_path,
        selection_tsv_path,
        receipt_path,
        selected_count: receipt.selected_count,
        extraction_executed: false,
        raw_to_rdf_promotion_allowed: false,
        validation_mode: receipt.validation_mode,
    })
}

/// Read selected source-contract `file_id` values from a generated selection
/// TSV artifact.
///
/// # Errors
///
/// Returns an error when the selection TSV cannot be read, has an unexpected
/// header or row width, has an invalid selection index, or contains duplicate
/// file ids.
pub fn read_episteme_evidence_selection_file_ids(
    selection_tsv_path: impl AsRef<Path>,
) -> Result<Vec<String>, EpistemeError> {
    let path = selection_tsv_path.as_ref();
    let raw = fs::read_to_string(path).map_err(|source| EpistemeError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let rows = parse_selection_tsv(path, raw.as_str())?;
    let file_ids = selected_file_ids_from_rows(rows)?;
    if file_ids.is_empty() {
        return Err(EpistemeError::EmptySelection);
    }
    Ok(file_ids)
}

fn selected_file_ids_from_rows(
    rows: Vec<EpistemeEvidenceSelectionRow>,
) -> Result<Vec<String>, EpistemeError> {
    let (file_ids, (_, errors)) = rows.into_iter().fold(
        (Vec::new(), (BTreeSet::new(), Vec::new())),
        |(mut file_ids, (mut seen, mut errors)), row| {
            if !seen.insert(row.file_id.clone()) {
                errors.push(format!("duplicate selected file_id: {}", row.file_id));
            }
            file_ids.push(row.file_id);
            (file_ids, (seen, errors))
        },
    );
    if errors.is_empty() {
        Ok(file_ids)
    } else {
        Err(EpistemeError::InvalidContract(errors))
    }
}

fn plan_episteme_evidence_selection(
    request: &EpistemeEvidenceSelectionPlanRequest,
) -> Result<EpistemeEvidenceSelectionReceipt, EpistemeError> {
    safe_run_id(request.run_id.as_str())?;
    if request.file_ids.is_empty() {
        return Err(EpistemeError::EmptySelection);
    }

    let manifest = read_source_manifest(&request.episteme_root)?;
    let paths = source_contract_paths(&request.episteme_root)?;
    let corpus_dir = paths.corpus_dir(&request.episteme_root)?;
    let files = read_files_tsv(&corpus_dir.join(&manifest.files))?;
    validate_for_mode(request, &manifest, &files)?;
    let rows = select_rows(&request.file_ids, &files, &request.selection_reason)?;

    Ok(EpistemeEvidenceSelectionReceipt {
        schema_version: SELECTION_RECEIPT_SCHEMA_VERSION,
        run_id: request.run_id.clone(),
        domain: manifest.domain,
        primary_language: manifest.primary_language,
        source_file_count: files.len(),
        selected_count: rows.len(),
        route_counts: count_by(rows.iter().map(|row| row.extraction_route.as_str())),
        category_counts: count_by(rows.iter().map(|row| row.category.as_str())),
        output_contract: EVIDENCE_SELECTION_OUTPUT_CONTRACT.to_string(),
        raw_to_rdf_promotion_allowed: false,
        extraction_executed: false,
        validation_mode: request.validation_mode,
        selection_reason: request.selection_reason.clone(),
        selections: rows,
    })
}

fn validate_for_mode(
    request: &EpistemeEvidenceSelectionPlanRequest,
    manifest: &EpistemeSourceManifest,
    files: &[EpistemeFileRow],
) -> Result<(), EpistemeError> {
    let errors = match request.validation_mode {
        EpistemeEvidenceSelectionValidationMode::FullHash => {
            let validation =
                validate_episteme_source_contract(&request.episteme_root, &request.corpus_root)?;
            if validation.passed {
                Vec::new()
            } else {
                validation.errors
            }
        }
        EpistemeEvidenceSelectionValidationMode::MetadataOnly => {
            validate_source_contract_metadata_only(&request.corpus_root, manifest, files)?
        }
    };
    if errors.is_empty() {
        Ok(())
    } else {
        Err(EpistemeError::InvalidContract(errors))
    }
}

fn select_rows(
    file_ids: &[String],
    files: &[EpistemeFileRow],
    selection_reason: &str,
) -> Result<Vec<EpistemeEvidenceSelectionRow>, EpistemeError> {
    let files_by_id = files
        .iter()
        .map(|row| (row.file_id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    let mut errors = Vec::new();
    for file_id in file_ids {
        if !seen.insert(file_id.as_str()) {
            errors.push(format!("duplicate selected file_id: {file_id}"));
        }
        if !files_by_id.contains_key(file_id.as_str()) {
            errors.push(format!("unknown selected file_id: {file_id}"));
        }
    }
    if !errors.is_empty() {
        return Err(EpistemeError::InvalidContract(errors));
    }

    file_ids
        .iter()
        .enumerate()
        .map(|(index, file_id)| {
            let source = files_by_id.get(file_id.as_str()).ok_or_else(|| {
                EpistemeError::InvalidContract(vec![format!("unknown selected file_id: {file_id}")])
            })?;
            Ok(selection_row(index + 1, source, selection_reason))
        })
        .collect()
}

fn selection_row(
    selection_index: usize,
    source: &EpistemeFileRow,
    selection_reason: &str,
) -> EpistemeEvidenceSelectionRow {
    EpistemeEvidenceSelectionRow {
        selection_index,
        file_id: source.file_id.clone(),
        relative_path: source.relative_path.clone(),
        extension: source.extension.clone(),
        byte_size: source.byte_size,
        source_sha256: source.sha256.clone(),
        category: source.category.clone(),
        language: source.language.clone(),
        extraction_route: source.extraction_route.clone(),
        selection_reason: selection_reason.to_string(),
        next_action: format!("extractor:{}", source.extraction_route),
    }
}

fn create_dir_all(path: &Path) -> Result<(), EpistemeError> {
    fs::create_dir_all(path).map_err(|source| EpistemeError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn write_selection_org(
    path: &Path,
    receipt: &EpistemeEvidenceSelectionReceipt,
) -> Result<(), EpistemeError> {
    let mut file = fs::File::create(path).map_err(|source| EpistemeError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    write_selection_org_header(file.by_ref(), path, receipt)?;
    write_selection_org_summary(file.by_ref(), path, receipt)?;
    write_selection_org_rows(file.by_ref(), path, &receipt.selections)
}

fn write_selection_org_header(
    mut writer: impl Write,
    path: &Path,
    receipt: &EpistemeEvidenceSelectionReceipt,
) -> Result<(), EpistemeError> {
    write_line(
        writer.by_ref(),
        path,
        "#+TITLE: Episteme Evidence Selection Plan",
    )?;
    write_line(writer.by_ref(), path, "#+OPTIONS: toc:nil")?;
    write_line(writer.by_ref(), path, "")?;
    write_line(writer.by_ref(), path, "* Evidence selection")?;
    write_line(writer.by_ref(), path, ":PROPERTIES:")?;
    write_line(
        writer.by_ref(),
        path,
        format!(
            ":ID: {}",
            deterministic_uuid("evidence-selection", receipt.run_id.as_str())
        )
        .as_str(),
    )?;
    write_line(
        writer.by_ref(),
        path,
        ":WENDAO_KIND: episteme_evidence_selection",
    )?;
    write_line(
        writer.by_ref(),
        path,
        ":ONTOLOGY_KIND: source_evidence_selection",
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
        "This ledger records selected source-contract file ids only. It does not embed raw corpus text, execute extraction, or promote RDF truth.",
    )
}

fn write_selection_org_summary(
    mut writer: impl Write,
    path: &Path,
    receipt: &EpistemeEvidenceSelectionReceipt,
) -> Result<(), EpistemeError> {
    write_line(writer.by_ref(), path, "")?;
    write_line(writer.by_ref(), path, "** Selection summary")?;
    write_line(writer.by_ref(), path, "| key | value |")?;
    write_line(writer.by_ref(), path, "|---+---|")?;
    write_line(
        writer.by_ref(),
        path,
        format!("| selected_count | {} |", receipt.selected_count).as_str(),
    )?;
    write_line(
        writer.by_ref(),
        path,
        format!(
            "| selection_reason | {} |",
            org_cell(&receipt.selection_reason)
        )
        .as_str(),
    )
}

fn write_selection_org_rows(
    mut writer: impl Write,
    path: &Path,
    rows: &[EpistemeEvidenceSelectionRow],
) -> Result<(), EpistemeError> {
    write_line(writer.by_ref(), path, "")?;
    write_line(writer.by_ref(), path, "** Selected evidence")?;
    write_line(
        writer.by_ref(),
        path,
        "| selection_index | file_id | relative_path | extension | byte_size | sha256 | category | language | extraction_route | selection_reason | next_action |",
    )?;
    write_line(
        writer.by_ref(),
        path,
        "|---+---+---+---+---+---+---+---+---+---+---|",
    )?;
    for row in rows {
        write_line(
            writer.by_ref(),
            path,
            format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
                row.selection_index,
                org_cell(&row.file_id),
                org_cell(&row.relative_path),
                org_cell(&row.extension),
                row.byte_size,
                org_cell(&row.source_sha256),
                org_cell(&row.category),
                org_cell(&row.language),
                org_cell(&row.extraction_route),
                org_cell(&row.selection_reason),
                org_cell(&row.next_action),
            )
            .as_str(),
        )?;
    }
    Ok(())
}

fn write_selection_tsv(
    path: &Path,
    selections: &[EpistemeEvidenceSelectionRow],
) -> Result<(), EpistemeError> {
    let mut file = fs::File::create(path).map_err(|source| EpistemeError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    writeln!(
        file,
        "selection_index\tfile_id\trelative_path\textension\tbyte_size\tsha256\tcategory\tlanguage\textraction_route\tselection_reason\tnext_action"
    )
    .map_err(|source| EpistemeError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    for row in selections {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            row.selection_index,
            tsv_cell(&row.file_id),
            tsv_cell(&row.relative_path),
            tsv_cell(&row.extension),
            row.byte_size,
            tsv_cell(&row.source_sha256),
            tsv_cell(&row.category),
            tsv_cell(&row.language),
            tsv_cell(&row.extraction_route),
            tsv_cell(&row.selection_reason),
            tsv_cell(&row.next_action),
        )
        .map_err(|source| EpistemeError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    }
    Ok(())
}

fn parse_selection_tsv(
    path: &Path,
    raw: &str,
) -> Result<Vec<EpistemeEvidenceSelectionRow>, EpistemeError> {
    let mut lines = raw.lines();
    let Some(header) = lines.next() else {
        return Err(EpistemeError::InvalidContract(vec![format!(
            "empty evidence selection TSV: {}",
            path.display()
        )]));
    };
    let actual = header.split('\t').collect::<Vec<_>>();
    if actual != SELECTION_TSV_FIELDS {
        return Err(EpistemeError::InvalidContract(vec![format!(
            "evidence selection TSV header mismatch: expected {:?}, got {:?}",
            SELECTION_TSV_FIELDS, actual
        )]));
    }
    lines
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| parse_selection_row(path, index + 2, line))
        .collect()
}

fn parse_selection_row(
    path: &Path,
    row_number: usize,
    line: &str,
) -> Result<EpistemeEvidenceSelectionRow, EpistemeError> {
    let fields = line.split('\t').collect::<Vec<_>>();
    if fields.len() != SELECTION_TSV_FIELDS.len() {
        return Err(EpistemeError::InvalidContract(vec![format!(
            "evidence selection TSV row {row_number} in {} has {} fields, expected {}",
            path.display(),
            fields.len(),
            SELECTION_TSV_FIELDS.len()
        )]));
    }
    let selection_index = fields[0].parse::<usize>().map_err(|source| {
        EpistemeError::InvalidContract(vec![format!(
            "invalid evidence selection index `{}` at row {row_number}: {source}",
            fields[0]
        )])
    })?;
    let byte_size = fields[4].parse::<u64>().map_err(|source| {
        EpistemeError::InvalidContract(vec![format!(
            "invalid evidence selection byte_size `{}` at row {row_number}: {source}",
            fields[4]
        )])
    })?;
    Ok(EpistemeEvidenceSelectionRow {
        selection_index,
        file_id: fields[1].to_string(),
        relative_path: fields[2].to_string(),
        extension: fields[3].to_string(),
        byte_size,
        source_sha256: fields[5].to_string(),
        category: fields[6].to_string(),
        language: fields[7].to_string(),
        extraction_route: fields[8].to_string(),
        selection_reason: fields[9].to_string(),
        next_action: fields[10].to_string(),
    })
}

fn write_receipt_json(
    path: &Path,
    receipt: &EpistemeEvidenceSelectionReceipt,
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

fn tsv_cell(value: &str) -> String {
    value.replace(['\n', '\r', '\t'], " ")
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
