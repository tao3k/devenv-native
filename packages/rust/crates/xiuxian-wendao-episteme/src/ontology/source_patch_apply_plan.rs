//! Non-mutating apply-plan generation from source-patch draft receipts.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const SOURCE_PATCH_APPLY_PLAN_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_source_patch_apply_plan.v1";
const SOURCE_PATCH_PREFLIGHT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_source_patch_preflight.v1";
const SOURCE_PATCH_DRAFT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_source_patch_draft.v1";
const SOURCE_PATCH_PREFLIGHT_TSV: &str = "source_patch_preflight.tsv";
const SOURCE_PATCH_PREFLIGHT_JSON: &str = "source_patch_preflight.json";
const SOURCE_PATCH_DRAFT_JSON: &str = "source_patch_draft.json";
const SOURCE_PATCH_APPLY_PLAN_TSV: &str = "source_patch_apply_plan.tsv";
const SOURCE_PATCH_APPLY_PLAN_ORG: &str = "source_patch_apply_plan.org";
const SOURCE_PATCH_APPLY_PLAN_JSON: &str = "source_patch_apply_plan.json";
const PREFLIGHT_ACTION_PROPOSE_SOURCE_PATCH: &str = "propose_source_patch";
const APPLY_ACTION_PROPOSE_TARGETED_SOURCE_PATCH: &str = "propose_targeted_source_patch";
const APPROVED_PROMOTION_DECISION: &str = "approved";
const OBJECT_INSTANCE_KIND: &str = "object_instance";
const INSTANCE_RELATION_KIND: &str = "instance_relation";
const SOURCE_MUTATION_ALLOWED: bool = false;
const ONTOLOGY_TRUTH: bool = false;

/// Request for writing a non-mutating source-patch apply plan.
#[derive(Debug, Clone)]
pub struct EpistemeOntologySourcePatchApplyPlanRequest {
    run_dir: PathBuf,
}

impl EpistemeOntologySourcePatchApplyPlanRequest {
    /// Create a source-patch apply-plan request from a source-patch run.
    #[must_use]
    pub fn new(run_dir: impl Into<PathBuf>) -> Self {
        Self {
            run_dir: run_dir.into(),
        }
    }

    /// Source-patch run directory containing preflight and draft receipts.
    #[must_use]
    pub fn run_dir(&self) -> &Path {
        self.run_dir.as_path()
    }
}

/// Report emitted after source-patch apply-plan generation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologySourcePatchApplyPlanReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Source-patch run directory.
    pub run_dir: PathBuf,
    /// Source preflight TSV path.
    pub source_patch_preflight_tsv: PathBuf,
    /// Source patch draft JSON path.
    pub source_patch_draft_json: PathBuf,
    /// Generated source-patch apply-plan TSV path.
    pub source_patch_apply_plan_tsv: PathBuf,
    /// Generated source-patch apply-plan Org path.
    pub source_patch_apply_plan_org: PathBuf,
    /// Generated source-patch apply-plan JSON path.
    pub source_patch_apply_plan_json: PathBuf,
    /// Number of preflight rows read.
    pub preflight_row_count: usize,
    /// Number of draft resources reported by the draft receipt.
    pub draft_resource_count: usize,
    /// Number of object-instance apply-plan rows.
    pub object_apply_plan_count: usize,
    /// Number of instance-relation apply-plan rows.
    pub relation_apply_plan_count: usize,
    /// Number of rows written to the apply plan.
    pub apply_plan_row_count: usize,
    /// Whether this plan authorizes source mutation.
    pub source_mutation_allowed: bool,
    /// Whether these rows are ontology truth.
    pub ontology_truth: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourcePatchPreflightReceipt {
    schema_version: String,
    preflight_row_count: usize,
    source_mutation_allowed: bool,
    ontology_truth: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourcePatchDraftReceipt {
    schema_version: String,
    preflight_row_count: usize,
    draft_resource_count: usize,
    source_mutation_allowed: bool,
    ontology_truth: bool,
}

#[derive(Debug, Clone)]
struct SourcePatchRow {
    record_id: String,
    record_kind: String,
    domain_id: String,
    target_rdf_file: String,
    label: String,
    object_type: String,
    source_object_id: String,
    predicate: String,
    target_object_id: String,
    evidence_id: String,
    review_decision: String,
    promotion_decision: String,
    reviewer_id: String,
    preflight_action: String,
    source_mutation_allowed: bool,
    ontology_truth: bool,
}

/// Write a non-mutating source-patch apply plan from draft-verified rows.
///
/// # Errors
///
/// Returns an error when preflight or draft receipts are missing, inconsistent,
/// unsafe, or when source-patch rows lack approved review and explicit target
/// metadata.
pub fn write_episteme_ontology_source_patch_apply_plan(
    request: &EpistemeOntologySourcePatchApplyPlanRequest,
) -> Result<EpistemeOntologySourcePatchApplyPlanReport> {
    write_episteme_ontology_source_patch_apply_plan_impl(request)
}

fn write_episteme_ontology_source_patch_apply_plan_impl(
    request: &EpistemeOntologySourcePatchApplyPlanRequest,
) -> Result<EpistemeOntologySourcePatchApplyPlanReport> {
    let run_dir = request.run_dir();
    let source_patch_preflight_tsv = run_dir.join(SOURCE_PATCH_PREFLIGHT_TSV);
    let source_patch_draft_json = run_dir.join(SOURCE_PATCH_DRAFT_JSON);
    let preflight_receipt =
        read_preflight_receipt(run_dir.join(SOURCE_PATCH_PREFLIGHT_JSON).as_path())?;
    let draft_receipt = read_draft_receipt(source_patch_draft_json.as_path())?;
    let rows = read_source_patch_rows(source_patch_preflight_tsv.as_path())?;
    validate_receipts(&preflight_receipt, &draft_receipt, rows.len())?;
    validate_source_patch_rows(&rows)?;

    let source_patch_apply_plan_tsv = run_dir.join(SOURCE_PATCH_APPLY_PLAN_TSV);
    let source_patch_apply_plan_org = run_dir.join(SOURCE_PATCH_APPLY_PLAN_ORG);
    let source_patch_apply_plan_json = run_dir.join(SOURCE_PATCH_APPLY_PLAN_JSON);
    write_apply_plan_tsv(source_patch_apply_plan_tsv.as_path(), &rows)?;

    let report = EpistemeOntologySourcePatchApplyPlanReport {
        schema_version: SOURCE_PATCH_APPLY_PLAN_SCHEMA_VERSION,
        run_dir: run_dir.to_path_buf(),
        source_patch_preflight_tsv,
        source_patch_draft_json,
        source_patch_apply_plan_tsv,
        source_patch_apply_plan_org,
        source_patch_apply_plan_json,
        preflight_row_count: rows.len(),
        draft_resource_count: draft_receipt.draft_resource_count,
        object_apply_plan_count: rows
            .iter()
            .filter(|row| row.record_kind == OBJECT_INSTANCE_KIND)
            .count(),
        relation_apply_plan_count: rows
            .iter()
            .filter(|row| row.record_kind == INSTANCE_RELATION_KIND)
            .count(),
        apply_plan_row_count: rows.len(),
        source_mutation_allowed: SOURCE_MUTATION_ALLOWED,
        ontology_truth: ONTOLOGY_TRUTH,
    };
    write_apply_plan_org(report.source_patch_apply_plan_org.as_path(), &report)?;
    write_json(report.source_patch_apply_plan_json.as_path(), &report)?;
    Ok(report)
}

fn read_preflight_receipt(path: &Path) -> Result<SourcePatchPreflightReceipt> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    serde_json::from_str(&content).with_context(|| {
        format!(
            "failed to parse source-patch preflight JSON `{}`",
            path.display()
        )
    })
}

fn read_draft_receipt(path: &Path) -> Result<SourcePatchDraftReceipt> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    serde_json::from_str(&content).with_context(|| {
        format!(
            "failed to parse source-patch draft JSON `{}`",
            path.display()
        )
    })
}

fn validate_receipts(
    preflight: &SourcePatchPreflightReceipt,
    draft: &SourcePatchDraftReceipt,
    row_count: usize,
) -> Result<()> {
    if preflight.schema_version != SOURCE_PATCH_PREFLIGHT_SCHEMA_VERSION {
        anyhow::bail!(
            "source-patch preflight receipt has unsupported schemaVersion `{}`",
            preflight.schema_version
        );
    }
    if draft.schema_version != SOURCE_PATCH_DRAFT_SCHEMA_VERSION {
        anyhow::bail!(
            "source-patch draft receipt has unsupported schemaVersion `{}`",
            draft.schema_version
        );
    }
    if preflight.source_mutation_allowed || draft.source_mutation_allowed {
        anyhow::bail!("source-patch receipts attempted to authorize source mutation");
    }
    if preflight.ontology_truth || draft.ontology_truth {
        anyhow::bail!("source-patch receipts attempted to mark ontology truth");
    }
    if preflight.preflight_row_count != row_count {
        anyhow::bail!(
            "source-patch preflight row count mismatch: receipt has {}, TSV has {row_count}",
            preflight.preflight_row_count
        );
    }
    if draft.preflight_row_count != row_count {
        anyhow::bail!(
            "source-patch draft preflight row count mismatch: receipt has {}, TSV has {row_count}",
            draft.preflight_row_count
        );
    }
    if draft.draft_resource_count != row_count {
        anyhow::bail!(
            "source-patch draft resource count mismatch: receipt has {}, TSV has {row_count}",
            draft.draft_resource_count
        );
    }
    Ok(())
}

fn read_source_patch_rows(path: &Path) -> Result<Vec<SourcePatchRow>> {
    let file = File::open(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let mut lines = BufReader::new(file).lines();
    let header = lines
        .next()
        .transpose()
        .with_context(|| format!("failed to read `{}`", path.display()))?
        .with_context(|| format!("source-patch preflight TSV `{}` is empty", path.display()))?;
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
                "source-patch preflight TSV `{}` row {} has {} values for {} columns",
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
        rows.push(source_patch_row(path, line_index + 2, &row)?);
    }
    Ok(rows)
}

fn source_patch_row(
    path: &Path,
    row_number: usize,
    row: &BTreeMap<String, String>,
) -> Result<SourcePatchRow> {
    Ok(SourcePatchRow {
        record_id: required(row, "record_id", path, row_number)?,
        record_kind: required(row, "record_kind", path, row_number)?,
        domain_id: required(row, "domain_id", path, row_number)?,
        target_rdf_file: required(row, "target_rdf_file", path, row_number)?,
        label: optional(row, "label"),
        object_type: optional(row, "object_type"),
        source_object_id: optional(row, "source_object_id"),
        predicate: optional(row, "predicate"),
        target_object_id: optional(row, "target_object_id"),
        evidence_id: required(row, "evidence_id", path, row_number)?,
        review_decision: required(row, "review_decision", path, row_number)?,
        promotion_decision: required(row, "promotion_decision", path, row_number)?,
        reviewer_id: required(row, "reviewer_id", path, row_number)?,
        preflight_action: required(row, "preflight_action", path, row_number)?,
        source_mutation_allowed: parse_bool(row, "source_mutation_allowed", path, row_number)?,
        ontology_truth: parse_bool(row, "ontology_truth", path, row_number)?,
    })
}

fn validate_source_patch_rows(rows: &[SourcePatchRow]) -> Result<()> {
    let mut object_ids = BTreeSet::new();
    for row in rows {
        if row.preflight_action != PREFLIGHT_ACTION_PROPOSE_SOURCE_PATCH {
            anyhow::bail!(
                "source-patch row `{}` has unsupported preflight_action `{}`",
                row.record_id,
                row.preflight_action
            );
        }
        if normalize(row.promotion_decision.as_str()) != APPROVED_PROMOTION_DECISION {
            anyhow::bail!(
                "source-patch row `{}` is not explicitly approved",
                row.record_id
            );
        }
        if row.source_mutation_allowed {
            anyhow::bail!(
                "source-patch row `{}` attempted to authorize source mutation",
                row.record_id
            );
        }
        if row.ontology_truth {
            anyhow::bail!(
                "source-patch row `{}` attempted to mark ontology truth",
                row.record_id
            );
        }
        require_nonblank(row.domain_id.as_str(), row.record_id.as_str(), "domain_id")?;
        require_nonblank(
            row.target_rdf_file.as_str(),
            row.record_id.as_str(),
            "target_rdf_file",
        )?;
        match row.record_kind.as_str() {
            OBJECT_INSTANCE_KIND => {
                require_nonblank(
                    row.object_type.as_str(),
                    row.record_id.as_str(),
                    "object_type",
                )?;
                require_nonblank(row.label.as_str(), row.record_id.as_str(), "label")?;
                if !object_ids.insert(row.record_id.as_str()) {
                    anyhow::bail!(
                        "source-patch apply plan contains duplicate object record `{}`",
                        row.record_id
                    );
                }
            }
            INSTANCE_RELATION_KIND => {
                require_nonblank(
                    row.source_object_id.as_str(),
                    row.record_id.as_str(),
                    "source_object_id",
                )?;
                require_nonblank(row.predicate.as_str(), row.record_id.as_str(), "predicate")?;
                require_nonblank(
                    row.target_object_id.as_str(),
                    row.record_id.as_str(),
                    "target_object_id",
                )?;
            }
            _ => anyhow::bail!(
                "source-patch row `{}` has unsupported record_kind `{}`",
                row.record_id,
                row.record_kind
            ),
        }
    }
    for row in rows
        .iter()
        .filter(|row| row.record_kind == INSTANCE_RELATION_KIND)
    {
        if !object_ids.contains(row.source_object_id.as_str()) {
            anyhow::bail!(
                "source-patch relation `{}` references source_object_id `{}` without an object apply-plan row",
                row.record_id,
                row.source_object_id
            );
        }
        if !object_ids.contains(row.target_object_id.as_str()) {
            anyhow::bail!(
                "source-patch relation `{}` references target_object_id `{}` without an object apply-plan row",
                row.record_id,
                row.target_object_id
            );
        }
    }
    Ok(())
}

fn write_apply_plan_tsv(path: &Path, rows: &[SourcePatchRow]) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "record_id\trecord_kind\tdomain_id\ttarget_rdf_file\tlabel\tobject_type\tsource_object_id\tpredicate\ttarget_object_id\tevidence_id\treview_decision\tpromotion_decision\treviewer_id\tapply_action\tsource_mutation_allowed\tontology_truth"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            escape_tsv(&row.record_id),
            escape_tsv(&row.record_kind),
            escape_tsv(&row.domain_id),
            escape_tsv(&row.target_rdf_file),
            escape_tsv(&row.label),
            escape_tsv(&row.object_type),
            escape_tsv(&row.source_object_id),
            escape_tsv(&row.predicate),
            escape_tsv(&row.target_object_id),
            escape_tsv(&row.evidence_id),
            escape_tsv(&row.review_decision),
            escape_tsv(&row.promotion_decision),
            escape_tsv(&row.reviewer_id),
            APPLY_ACTION_PROPOSE_TARGETED_SOURCE_PATCH,
            SOURCE_MUTATION_ALLOWED,
            ONTOLOGY_TRUTH
        )?;
    }
    Ok(())
}

fn write_apply_plan_org(
    path: &Path,
    report: &EpistemeOntologySourcePatchApplyPlanReport,
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(file, "#+TITLE: Ontology Source Patch Apply Plan")?;
    writeln!(file)?;
    writeln!(file, "* Source patch apply plan")?;
    writeln!(file, ":PROPERTIES:")?;
    writeln!(file, ":WENDAO_KIND: ontology_source_patch_apply_plan")?;
    writeln!(file, ":SOURCE_MUTATION_ALLOWED: false")?;
    writeln!(file, ":ONTOLOGY_TRUTH: false")?;
    writeln!(file, ":END:")?;
    writeln!(file)?;
    writeln!(
        file,
        "This apply plan is a non-mutating review surface for targeted ontology source patches."
    )?;
    writeln!(file)?;
    writeln!(file, "| field | value |")?;
    writeln!(file, "|-|-|")?;
    writeln!(
        file,
        "| preflight_row_count | {} |",
        report.preflight_row_count
    )?;
    writeln!(
        file,
        "| draft_resource_count | {} |",
        report.draft_resource_count
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
        "| apply_plan_row_count | {} |",
        report.apply_plan_row_count
    )?;
    writeln!(
        file,
        "| source_mutation_allowed | {} |",
        report.source_mutation_allowed
    )?;
    writeln!(file, "| ontology_truth | {} |", report.ontology_truth)?;
    Ok(())
}

fn write_json(path: &Path, report: &EpistemeOntologySourcePatchApplyPlanReport) -> Result<()> {
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
            "source-patch preflight TSV `{}` row {row_number} missing `{name}` column",
            path.display()
        )
    })
}

fn optional(row: &BTreeMap<String, String>, name: &str) -> String {
    row.get(name).cloned().unwrap_or_default()
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
            "source-patch preflight TSV `{}` row {row_number} has invalid `{name}` value `{value}`",
            path.display()
        ),
    }
}

fn require_nonblank(value: &str, record_id: &str, field: &str) -> Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("source-patch row `{record_id}` must declare nonblank {field}");
    }
    Ok(())
}

fn normalize(value: &str) -> String {
    value.trim().replace(['-', ' '], "_").to_lowercase()
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

fn escape_tsv(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
