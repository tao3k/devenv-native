//! Hash-guarded review packet generation from source-patch apply plans.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{BufRead, BufReader, Read, Write},
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const SOURCE_PATCH_REVIEW_PACKET_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_source_patch_review_packet.v1";
const SOURCE_PATCH_APPLY_PLAN_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_source_patch_apply_plan.v1";
const SOURCE_PATCH_APPLY_PLAN_TSV: &str = "source_patch_apply_plan.tsv";
const SOURCE_PATCH_APPLY_PLAN_JSON: &str = "source_patch_apply_plan.json";
const SOURCE_PATCH_REVIEW_PACKET_ORG: &str = "source_patch_review_packet.org";
const SOURCE_PATCH_REVIEW_PACKET_JSON: &str = "source_patch_review_packet.json";
const APPLY_ACTION_PROPOSE_TARGETED_SOURCE_PATCH: &str = "propose_targeted_source_patch";
const SOURCE_MUTATION_ALLOWED: bool = false;
const ONTOLOGY_TRUTH: bool = false;

/// Request for writing a hash-guarded source-patch review packet.
#[derive(Debug, Clone)]
pub struct EpistemeOntologySourcePatchReviewPacketRequest {
    episteme_root: PathBuf,
    run_dir: PathBuf,
}

impl EpistemeOntologySourcePatchReviewPacketRequest {
    /// Create a source-patch review-packet request.
    #[must_use]
    pub fn new(episteme_root: impl Into<PathBuf>, run_dir: impl Into<PathBuf>) -> Self {
        Self {
            episteme_root: episteme_root.into(),
            run_dir: run_dir.into(),
        }
    }

    /// Episteme repository root containing ontology source files.
    #[must_use]
    pub fn episteme_root(&self) -> &Path {
        self.episteme_root.as_path()
    }

    /// Source-patch run directory containing apply-plan artifacts.
    #[must_use]
    pub fn run_dir(&self) -> &Path {
        self.run_dir.as_path()
    }
}

/// Hash metadata for one target RDF source file referenced by a review packet.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologySourcePatchReviewPacketTarget {
    /// Target RDF path relative to the Episteme `ontology/` directory.
    pub target_rdf_file: String,
    /// Current SHA-256 digest of the target RDF file.
    pub target_rdf_sha256: String,
}

/// Report emitted after source-patch review-packet generation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologySourcePatchReviewPacketReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Episteme repository root used to resolve target RDF files.
    pub episteme_root: PathBuf,
    /// Source-patch run directory.
    pub run_dir: PathBuf,
    /// Source apply-plan TSV path.
    pub source_patch_apply_plan_tsv: PathBuf,
    /// Source apply-plan JSON path.
    pub source_patch_apply_plan_json: PathBuf,
    /// Generated review-packet Org path.
    pub source_patch_review_packet_org: PathBuf,
    /// Generated review-packet JSON path.
    pub source_patch_review_packet_json: PathBuf,
    /// Stable SHA-256 digest for the apply-plan TSV.
    pub apply_plan_tsv_sha256: String,
    /// Number of apply-plan rows read from TSV.
    pub apply_plan_row_count: usize,
    /// Number of object-instance apply-plan rows.
    pub object_apply_plan_count: usize,
    /// Number of instance-relation apply-plan rows.
    pub relation_apply_plan_count: usize,
    /// Number of unique target RDF files referenced by the plan.
    pub target_rdf_file_count: usize,
    /// Hash metadata for every referenced target RDF file.
    pub target_rdf_files: Vec<EpistemeOntologySourcePatchReviewPacketTarget>,
    /// Whether this packet authorizes source mutation.
    pub source_mutation_allowed: bool,
    /// Whether these rows are ontology truth.
    pub ontology_truth: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourcePatchApplyPlanReceipt {
    schema_version: String,
    object_apply_plan_count: usize,
    relation_apply_plan_count: usize,
    apply_plan_row_count: usize,
    source_mutation_allowed: bool,
    ontology_truth: bool,
}

#[derive(Debug)]
struct SourcePatchApplyPlanRow {
    record_id: String,
    record_kind: String,
    target_rdf_file: String,
    apply_action: String,
    source_mutation_allowed: bool,
    ontology_truth: bool,
}

/// Write a hash-guarded source-patch review packet from an apply plan.
///
/// # Errors
///
/// Returns an error when apply-plan artifacts are missing, inconsistent,
/// unsafe, or when a referenced target RDF file is outside the Episteme root,
/// missing, or cannot be hashed.
pub fn write_episteme_ontology_source_patch_review_packet(
    request: &EpistemeOntologySourcePatchReviewPacketRequest,
) -> Result<EpistemeOntologySourcePatchReviewPacketReport> {
    write_episteme_ontology_source_patch_review_packet_impl(request)
}

fn write_episteme_ontology_source_patch_review_packet_impl(
    request: &EpistemeOntologySourcePatchReviewPacketRequest,
) -> Result<EpistemeOntologySourcePatchReviewPacketReport> {
    let episteme_root = request.episteme_root();
    let run_dir = request.run_dir();
    let apply_plan_tsv = run_dir.join(SOURCE_PATCH_APPLY_PLAN_TSV);
    let apply_plan_json = run_dir.join(SOURCE_PATCH_APPLY_PLAN_JSON);
    let receipt = read_apply_plan_receipt(apply_plan_json.as_path())?;
    let rows = read_apply_plan_rows(apply_plan_tsv.as_path())?;
    validate_apply_plan(&receipt, &rows)?;

    let target_rdf_files = target_rdf_hashes(episteme_root, &rows)?;
    let output_org = run_dir.join(SOURCE_PATCH_REVIEW_PACKET_ORG);
    let output_json = run_dir.join(SOURCE_PATCH_REVIEW_PACKET_JSON);
    let report = EpistemeOntologySourcePatchReviewPacketReport {
        schema_version: SOURCE_PATCH_REVIEW_PACKET_SCHEMA_VERSION,
        episteme_root: episteme_root.to_path_buf(),
        run_dir: run_dir.to_path_buf(),
        source_patch_apply_plan_tsv: apply_plan_tsv.clone(),
        source_patch_apply_plan_json: apply_plan_json,
        source_patch_review_packet_org: output_org,
        source_patch_review_packet_json: output_json,
        apply_plan_tsv_sha256: sha256_file(apply_plan_tsv.as_path())?,
        apply_plan_row_count: rows.len(),
        object_apply_plan_count: receipt.object_apply_plan_count,
        relation_apply_plan_count: receipt.relation_apply_plan_count,
        target_rdf_file_count: target_rdf_files.len(),
        target_rdf_files,
        source_mutation_allowed: SOURCE_MUTATION_ALLOWED,
        ontology_truth: ONTOLOGY_TRUTH,
    };
    write_review_packet_org(report.source_patch_review_packet_org.as_path(), &report)?;
    write_json(report.source_patch_review_packet_json.as_path(), &report)?;
    Ok(report)
}

fn read_apply_plan_receipt(path: &Path) -> Result<SourcePatchApplyPlanReceipt> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("failed to parse apply-plan JSON `{}`", path.display()))
}

fn read_apply_plan_rows(path: &Path) -> Result<Vec<SourcePatchApplyPlanRow>> {
    let file = File::open(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let mut lines = BufReader::new(file).lines();
    let header = lines
        .next()
        .transpose()
        .with_context(|| format!("failed to read `{}`", path.display()))?
        .with_context(|| format!("source-patch apply-plan TSV `{}` is empty", path.display()))?;
    let columns = header.split('\t').map(str::to_string).collect::<Vec<_>>();
    let mut rows = Vec::new();
    for (line_index, line) in lines.enumerate() {
        let line = line.with_context(|| format!("failed to read `{}`", path.display()))?;
        if line.trim().is_empty() {
            continue;
        }
        let values = line.split('\t').map(unescape_tsv).collect::<Vec<_>>();
        if values.len() != columns.len() {
            anyhow::bail!(
                "source-patch apply-plan TSV `{}` row {} has {} values for {} columns",
                path.display(),
                line_index + 2,
                values.len(),
                columns.len()
            );
        }
        let row = columns
            .iter()
            .cloned()
            .zip(values)
            .collect::<BTreeMap<_, _>>();
        rows.push(apply_plan_row(path, line_index + 2, &row)?);
    }
    Ok(rows)
}

fn apply_plan_row(
    path: &Path,
    row_number: usize,
    row: &BTreeMap<String, String>,
) -> Result<SourcePatchApplyPlanRow> {
    Ok(SourcePatchApplyPlanRow {
        record_id: required(row, "record_id", path, row_number)?,
        record_kind: required(row, "record_kind", path, row_number)?,
        target_rdf_file: required(row, "target_rdf_file", path, row_number)?,
        apply_action: required(row, "apply_action", path, row_number)?,
        source_mutation_allowed: parse_bool(row, "source_mutation_allowed", path, row_number)?,
        ontology_truth: parse_bool(row, "ontology_truth", path, row_number)?,
    })
}

fn validate_apply_plan(
    receipt: &SourcePatchApplyPlanReceipt,
    rows: &[SourcePatchApplyPlanRow],
) -> Result<()> {
    if receipt.schema_version != SOURCE_PATCH_APPLY_PLAN_SCHEMA_VERSION {
        anyhow::bail!(
            "source-patch apply-plan receipt has unsupported schemaVersion `{}`",
            receipt.schema_version
        );
    }
    if receipt.source_mutation_allowed {
        anyhow::bail!("source-patch apply-plan attempted to authorize source mutation");
    }
    if receipt.ontology_truth {
        anyhow::bail!("source-patch apply-plan attempted to mark ontology truth");
    }
    if receipt.apply_plan_row_count != rows.len() {
        anyhow::bail!(
            "source-patch apply-plan row count mismatch: receipt has {}, TSV has {}",
            receipt.apply_plan_row_count,
            rows.len()
        );
    }
    let object_count = rows
        .iter()
        .filter(|row| row.record_kind == "object_instance")
        .count();
    let relation_count = rows
        .iter()
        .filter(|row| row.record_kind == "instance_relation")
        .count();
    if receipt.object_apply_plan_count != object_count {
        anyhow::bail!(
            "source-patch object apply-plan count mismatch: receipt has {}, TSV has {object_count}",
            receipt.object_apply_plan_count
        );
    }
    if receipt.relation_apply_plan_count != relation_count {
        anyhow::bail!(
            "source-patch relation apply-plan count mismatch: receipt has {}, TSV has {relation_count}",
            receipt.relation_apply_plan_count
        );
    }
    for row in rows {
        if row.apply_action != APPLY_ACTION_PROPOSE_TARGETED_SOURCE_PATCH {
            anyhow::bail!(
                "source-patch apply-plan row `{}` has unsupported apply_action `{}`",
                row.record_id,
                row.apply_action
            );
        }
        if row.source_mutation_allowed {
            anyhow::bail!(
                "source-patch apply-plan row `{}` attempted to authorize source mutation",
                row.record_id
            );
        }
        if row.ontology_truth {
            anyhow::bail!(
                "source-patch apply-plan row `{}` attempted to mark ontology truth",
                row.record_id
            );
        }
    }
    Ok(())
}

fn target_rdf_hashes(
    episteme_root: &Path,
    rows: &[SourcePatchApplyPlanRow],
) -> Result<Vec<EpistemeOntologySourcePatchReviewPacketTarget>> {
    let target_files = rows
        .iter()
        .map(|row| row.target_rdf_file.as_str())
        .collect::<BTreeSet<_>>();
    let mut targets = Vec::new();
    for target in target_files {
        let path = resolve_target_rdf_file(episteme_root, target)?;
        targets.push(EpistemeOntologySourcePatchReviewPacketTarget {
            target_rdf_file: target.to_string(),
            target_rdf_sha256: sha256_file(path.as_path())?,
        });
    }
    Ok(targets)
}

fn resolve_target_rdf_file(episteme_root: &Path, target_rdf_file: &str) -> Result<PathBuf> {
    if target_rdf_file.trim().is_empty() {
        anyhow::bail!("source-patch review packet target RDF file is blank");
    }
    let relative = Path::new(target_rdf_file);
    if relative.is_absolute() || has_parent_dir(relative) {
        anyhow::bail!("source-patch target RDF file `{target_rdf_file}` is unsafe");
    }
    let root = episteme_root
        .canonicalize()
        .with_context(|| format!("failed to canonicalize `{}`", episteme_root.display()))?;
    let ontology_root = root.join("ontology");
    let path = ontology_root.join(relative);
    let canonical = path
        .canonicalize()
        .with_context(|| format!("failed to canonicalize `{}`", path.display()))?;
    if !canonical.starts_with(ontology_root) {
        anyhow::bail!("source-patch target RDF file `{target_rdf_file}` escapes ontology root");
    }
    Ok(canonical)
}

fn has_parent_dir(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::ParentDir))
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file =
        File::open(path).with_context(|| format!("failed to hash `{}`", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file
            .read(&mut buffer)
            .with_context(|| format!("failed to read `{}`", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex_digest(&hasher.finalize()))
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

fn write_review_packet_org(
    path: &Path,
    report: &EpistemeOntologySourcePatchReviewPacketReport,
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(file, "#+TITLE: Ontology Source Patch Review Packet")?;
    writeln!(file)?;
    writeln!(file, "* Source patch review packet")?;
    writeln!(file, ":PROPERTIES:")?;
    writeln!(file, ":WENDAO_KIND: ontology_source_patch_review_packet")?;
    writeln!(
        file,
        ":APPLY_PLAN_TSV_SHA256: {}",
        report.apply_plan_tsv_sha256
    )?;
    writeln!(file, ":SOURCE_MUTATION_ALLOWED: false")?;
    writeln!(file, ":ONTOLOGY_TRUTH: false")?;
    writeln!(file, ":END:")?;
    writeln!(file)?;
    writeln!(
        file,
        "This packet binds a source-patch apply plan to the current target RDF source hashes. It does not authorize source mutation."
    )?;
    writeln!(file)?;
    writeln!(file, "| field | value |")?;
    writeln!(file, "|-|-|")?;
    writeln!(
        file,
        "| apply_plan_row_count | {} |",
        report.apply_plan_row_count
    )?;
    writeln!(
        file,
        "| object_apply_plan_count | {} |",
        report.object_apply_plan_count
    )?;
    writeln!(
        file,
        "| relation_apply_plan_count | {} |",
        report.relation_apply_plan_count
    )?;
    writeln!(
        file,
        "| target_rdf_file_count | {} |",
        report.target_rdf_file_count
    )?;
    writeln!(file, "| source_mutation_allowed | false |")?;
    writeln!(file, "| ontology_truth | false |")?;
    writeln!(file)?;
    writeln!(file, "** Target RDF hashes")?;
    writeln!(file, "| target_rdf_file | target_rdf_sha256 |")?;
    writeln!(file, "|-|-|")?;
    for target in &report.target_rdf_files {
        writeln!(
            file,
            "| {} | {} |",
            target.target_rdf_file, target.target_rdf_sha256
        )?;
    }
    Ok(())
}

fn write_json(path: &Path, report: &EpistemeOntologySourcePatchReviewPacketReport) -> Result<()> {
    let mut file = create_file(path)?;
    serde_json::to_writer_pretty(&mut file, report)
        .with_context(|| format!("failed to write `{}`", path.display()))?;
    writeln!(file)?;
    Ok(())
}

fn create_file(path: &Path) -> Result<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create `{}`", parent.display()))?;
    }
    File::create(path).with_context(|| format!("failed to create `{}`", path.display()))
}

fn required(
    row: &BTreeMap<String, String>,
    name: &str,
    path: &Path,
    row_number: usize,
) -> Result<String> {
    row.get(name).cloned().with_context(|| {
        format!(
            "source-patch apply-plan TSV `{}` row {row_number} missing `{name}` column",
            path.display()
        )
    })
}

fn parse_bool(
    row: &BTreeMap<String, String>,
    name: &str,
    path: &Path,
    row_number: usize,
) -> Result<bool> {
    let value = required(row, name, path, row_number)?;
    match value.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => anyhow::bail!(
            "source-patch apply-plan TSV `{}` row {row_number} has invalid `{name}` value `{value}`",
            path.display()
        ),
    }
}

fn unescape_tsv(value: &str) -> String {
    let mut output = String::new();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('t') => output.push('\t'),
                Some('n') => output.push('\n'),
                Some('r') => output.push('\r'),
                Some('\\') | None => output.push('\\'),
                Some(other) => {
                    output.push('\\');
                    output.push(other);
                }
            }
        } else {
            output.push(ch);
        }
    }
    output
}
