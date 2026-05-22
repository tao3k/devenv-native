//! Promotion apply-plan generation from explicit ontology review decisions.

use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Serialize;

const PROMOTION_APPLY_PLAN_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_promotion_apply_plan.v1";
const PROMOTION_REVIEW_TSV: &str = "promotion_review.tsv";
const PROMOTION_APPLY_PLAN_TSV: &str = "promotion_apply_plan.tsv";
const PROMOTION_APPLY_PLAN_ORG: &str = "promotion_apply_plan.org";
const PROMOTION_APPLY_PLAN_JSON: &str = "promotion_apply_plan.json";
const APPLY_ACTION_PROPOSE_SOURCE_PATCH: &str = "propose_source_patch";
const SOURCE_MUTATION_ALLOWED: bool = false;
const ONTOLOGY_TRUTH: bool = false;

/// Request for writing a promotion apply plan from explicit review decisions.
#[derive(Debug, Clone)]
pub struct EpistemeOntologyPromotionApplyPlanRequest {
    run_dir: PathBuf,
}

impl EpistemeOntologyPromotionApplyPlanRequest {
    /// Create a promotion apply-plan request from an ontology-generation run directory.
    #[must_use]
    pub fn new(run_dir: impl Into<PathBuf>) -> Self {
        Self {
            run_dir: run_dir.into(),
        }
    }

    /// Ontology-generation run directory containing `promotion_review.tsv`.
    #[must_use]
    pub fn run_dir(&self) -> &Path {
        self.run_dir.as_path()
    }
}

/// Report emitted after promotion apply-plan generation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyPromotionApplyPlanReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Reviewed ontology-generation run directory.
    pub run_dir: PathBuf,
    /// Generated promotion apply-plan TSV path.
    pub promotion_apply_plan_tsv: PathBuf,
    /// Generated promotion apply-plan Org path.
    pub promotion_apply_plan_org: PathBuf,
    /// Generated promotion apply-plan JSON path.
    pub promotion_apply_plan_json: PathBuf,
    /// Number of promotion review rows read.
    pub promotion_review_row_count: usize,
    /// Number of rows still pending review.
    pub pending_review_count: usize,
    /// Number of explicitly approved rows.
    pub approved_count: usize,
    /// Number of explicitly rejected rows.
    pub rejected_count: usize,
    /// Number of rows that need stronger evidence.
    pub needs_evidence_count: usize,
    /// Number of rows written to the apply plan.
    pub apply_plan_row_count: usize,
    /// Whether source mutation is authorized by this plan.
    pub source_mutation_allowed: bool,
    /// Whether generated rows are ontology truth.
    pub ontology_truth: bool,
}

#[derive(Debug, Clone)]
struct PromotionReviewRecord {
    record_id: String,
    record_kind: String,
    label: String,
    suggested_term_key: String,
    review_decision: String,
    quality_score: usize,
    evidence_strength: String,
    issue_codes: String,
    promotion_precondition_met: bool,
    source_file_id: String,
    source_queue_id: String,
    extraction_run_id: String,
    relation_source_candidate_id: String,
    relation_target_candidate_id: String,
    promotion_decision: PromotionDecision,
    source_mutation_allowed: bool,
    ontology_truth: bool,
    reviewer_id: String,
    reviewer_note: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum PromotionDecision {
    PendingReview,
    Approved,
    Rejected,
    NeedsEvidence,
}

impl PromotionDecision {
    fn parse(value: &str, record_id: &str) -> Result<Self> {
        match value {
            "pending_review" => Ok(Self::PendingReview),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            "needs_evidence" => Ok(Self::NeedsEvidence),
            _ => anyhow::bail!(
                "promotion review row `{record_id}` has unknown promotion decision `{value}`"
            ),
        }
    }
}

#[derive(Debug, Clone)]
struct ApplyPlanRow {
    record_id: String,
    record_kind: String,
    label: String,
    suggested_term_key: String,
    relation_source_candidate_id: String,
    relation_target_candidate_id: String,
    review_decision: String,
    quality_score: usize,
    evidence_strength: String,
    issue_codes: String,
    source_file_id: String,
    source_queue_id: String,
    extraction_run_id: String,
    reviewer_id: String,
    reviewer_note: String,
    apply_action: &'static str,
    source_mutation_allowed: bool,
    ontology_truth: bool,
}

#[derive(Default)]
struct DecisionCounts {
    pending_review: usize,
    approved: usize,
    rejected: usize,
    needs_evidence: usize,
}

/// Write a promotion apply plan from explicit review decisions.
///
/// # Errors
///
/// Returns an error when `promotion_review.tsv` is missing, has an unknown
/// decision, contains unsafe mutation flags, or contains approved rows without
/// reviewer provenance.
pub fn write_episteme_ontology_promotion_apply_plan(
    request: &EpistemeOntologyPromotionApplyPlanRequest,
) -> Result<EpistemeOntologyPromotionApplyPlanReport> {
    let run_dir = request.run_dir();
    let review_rows = read_promotion_review(run_dir.join(PROMOTION_REVIEW_TSV).as_path())?;
    validate_review_rows(&review_rows)?;
    let counts = count_decisions(&review_rows);
    let plan_rows = build_apply_plan_rows(&review_rows);

    let promotion_apply_plan_tsv = run_dir.join(PROMOTION_APPLY_PLAN_TSV);
    let promotion_apply_plan_org = run_dir.join(PROMOTION_APPLY_PLAN_ORG);
    let promotion_apply_plan_json = run_dir.join(PROMOTION_APPLY_PLAN_JSON);
    write_apply_plan_tsv(promotion_apply_plan_tsv.as_path(), &plan_rows)?;

    let report = EpistemeOntologyPromotionApplyPlanReport {
        schema_version: PROMOTION_APPLY_PLAN_SCHEMA_VERSION,
        run_dir: run_dir.to_path_buf(),
        promotion_apply_plan_tsv,
        promotion_apply_plan_org,
        promotion_apply_plan_json,
        promotion_review_row_count: review_rows.len(),
        pending_review_count: counts.pending_review,
        approved_count: counts.approved,
        rejected_count: counts.rejected,
        needs_evidence_count: counts.needs_evidence,
        apply_plan_row_count: plan_rows.len(),
        source_mutation_allowed: SOURCE_MUTATION_ALLOWED,
        ontology_truth: ONTOLOGY_TRUTH,
    };
    write_apply_plan_org(report.promotion_apply_plan_org.as_path(), &report)?;
    write_json(report.promotion_apply_plan_json.as_path(), &report)?;
    Ok(report)
}

fn validate_review_rows(rows: &[PromotionReviewRecord]) -> Result<()> {
    for row in rows {
        if row.source_mutation_allowed {
            anyhow::bail!(
                "promotion review row `{}` attempted to authorize source mutation",
                row.record_id
            );
        }
        if row.ontology_truth {
            anyhow::bail!(
                "promotion review row `{}` attempted to mark ontology truth",
                row.record_id
            );
        }
        if row.promotion_decision == PromotionDecision::Approved {
            if !row.promotion_precondition_met {
                anyhow::bail!(
                    "approved promotion review row `{}` does not meet the promotion precondition",
                    row.record_id
                );
            }
            if row.reviewer_id.trim().is_empty() {
                anyhow::bail!(
                    "approved promotion review row `{}` requires reviewer provenance",
                    row.record_id
                );
            }
        }
    }
    Ok(())
}

fn count_decisions(rows: &[PromotionReviewRecord]) -> DecisionCounts {
    let mut counts = DecisionCounts::default();
    for row in rows {
        match row.promotion_decision {
            PromotionDecision::PendingReview => counts.pending_review += 1,
            PromotionDecision::Approved => counts.approved += 1,
            PromotionDecision::Rejected => counts.rejected += 1,
            PromotionDecision::NeedsEvidence => counts.needs_evidence += 1,
        }
    }
    counts
}

fn build_apply_plan_rows(rows: &[PromotionReviewRecord]) -> Vec<ApplyPlanRow> {
    rows.iter()
        .filter(|row| row.promotion_decision == PromotionDecision::Approved)
        .map(|row| ApplyPlanRow {
            record_id: row.record_id.clone(),
            record_kind: row.record_kind.clone(),
            label: row.label.clone(),
            suggested_term_key: row.suggested_term_key.clone(),
            relation_source_candidate_id: row.relation_source_candidate_id.clone(),
            relation_target_candidate_id: row.relation_target_candidate_id.clone(),
            review_decision: row.review_decision.clone(),
            quality_score: row.quality_score,
            evidence_strength: row.evidence_strength.clone(),
            issue_codes: row.issue_codes.clone(),
            source_file_id: row.source_file_id.clone(),
            source_queue_id: row.source_queue_id.clone(),
            extraction_run_id: row.extraction_run_id.clone(),
            reviewer_id: row.reviewer_id.clone(),
            reviewer_note: row.reviewer_note.clone(),
            apply_action: APPLY_ACTION_PROPOSE_SOURCE_PATCH,
            source_mutation_allowed: SOURCE_MUTATION_ALLOWED,
            ontology_truth: ONTOLOGY_TRUTH,
        })
        .collect()
}

fn read_promotion_review(path: &Path) -> Result<Vec<PromotionReviewRecord>> {
    let table = read_tsv(path)?;
    table
        .rows
        .iter()
        .map(|row| {
            let record_id = required(row, "record_id", path)?;
            let promotion_decision = PromotionDecision::parse(
                required(row, "promotion_decision", path)?.as_str(),
                record_id.as_str(),
            )?;
            Ok(PromotionReviewRecord {
                record_id,
                record_kind: required(row, "record_kind", path)?,
                label: optional(row, "label"),
                suggested_term_key: optional(row, "suggested_term_key"),
                review_decision: required(row, "review_decision", path)?,
                quality_score: parse_usize(row, "quality_score", path)?,
                evidence_strength: required(row, "evidence_strength", path)?,
                issue_codes: optional(row, "issue_codes"),
                promotion_precondition_met: parse_bool(row, "promotion_precondition_met", path)?,
                source_file_id: optional(row, "source_file_id"),
                source_queue_id: optional(row, "source_queue_id"),
                extraction_run_id: optional(row, "extraction_run_id"),
                relation_source_candidate_id: optional(row, "relation_source_candidate_id"),
                relation_target_candidate_id: optional(row, "relation_target_candidate_id"),
                promotion_decision,
                source_mutation_allowed: parse_bool(row, "source_mutation_allowed", path)?,
                ontology_truth: parse_bool(row, "ontology_truth", path)?,
                reviewer_id: optional(row, "reviewer_id"),
                reviewer_note: optional(row, "reviewer_note"),
            })
        })
        .collect()
}

struct TsvTable {
    rows: Vec<BTreeMap<String, String>>,
}

fn read_tsv(path: &Path) -> Result<TsvTable> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let mut lines = content.lines();
    let header_line = lines
        .next()
        .with_context(|| format!("promotion review TSV `{}` is empty", path.display()))?;
    let headers: Vec<String> = header_line.split('\t').map(ToString::to_string).collect();
    if headers.is_empty() {
        anyhow::bail!("promotion review TSV `{}` has no header", path.display());
    }
    let rows = lines
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| parse_tsv_row(path, &headers, index + 2, line))
        .collect::<Result<Vec<_>>>()?;
    Ok(TsvTable { rows })
}

fn parse_tsv_row(
    path: &Path,
    headers: &[String],
    line_number: usize,
    line: &str,
) -> Result<BTreeMap<String, String>> {
    let values: Vec<&str> = line.split('\t').collect();
    if values.len() != headers.len() {
        anyhow::bail!(
            "promotion review TSV `{}` row {} has {} columns, expected {}",
            path.display(),
            line_number,
            values.len(),
            headers.len()
        );
    }
    Ok(headers
        .iter()
        .cloned()
        .zip(values.into_iter().map(unescape_tsv))
        .collect())
}

fn required(row: &BTreeMap<String, String>, name: &str, path: &Path) -> Result<String> {
    row.get(name).cloned().with_context(|| {
        format!(
            "promotion review TSV `{}` missing `{name}` column",
            path.display()
        )
    })
}

fn optional(row: &BTreeMap<String, String>, name: &str) -> String {
    row.get(name).cloned().unwrap_or_default()
}

fn parse_usize(row: &BTreeMap<String, String>, name: &str, path: &Path) -> Result<usize> {
    let value = required(row, name, path)?;
    value.parse::<usize>().with_context(|| {
        format!(
            "promotion review TSV `{}` has invalid `{name}` value `{value}`",
            path.display()
        )
    })
}

fn parse_bool(row: &BTreeMap<String, String>, name: &str, path: &Path) -> Result<bool> {
    let value = required(row, name, path)?;
    match value.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => anyhow::bail!(
            "promotion review TSV `{}` has invalid `{name}` value `{value}`",
            path.display()
        ),
    }
}

fn write_apply_plan_tsv(path: &Path, rows: &[ApplyPlanRow]) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "record_id\trecord_kind\tlabel\tsuggested_term_key\trelation_source_candidate_id\trelation_target_candidate_id\treview_decision\tquality_score\tevidence_strength\tissue_codes\tsource_file_id\tsource_queue_id\textraction_run_id\treviewer_id\treviewer_note\tapply_action\tsource_mutation_allowed\tontology_truth"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            escape_tsv(&row.record_id),
            escape_tsv(&row.record_kind),
            escape_tsv(&row.label),
            escape_tsv(&row.suggested_term_key),
            escape_tsv(&row.relation_source_candidate_id),
            escape_tsv(&row.relation_target_candidate_id),
            escape_tsv(&row.review_decision),
            row.quality_score,
            escape_tsv(&row.evidence_strength),
            escape_tsv(&row.issue_codes),
            escape_tsv(&row.source_file_id),
            escape_tsv(&row.source_queue_id),
            escape_tsv(&row.extraction_run_id),
            escape_tsv(&row.reviewer_id),
            escape_tsv(&row.reviewer_note),
            row.apply_action,
            row.source_mutation_allowed,
            row.ontology_truth
        )?;
    }
    Ok(())
}

fn write_apply_plan_org(
    path: &Path,
    report: &EpistemeOntologyPromotionApplyPlanReport,
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(file, "#+TITLE: Private Ontology Promotion Apply Plan")?;
    writeln!(file)?;
    writeln!(file, "* Promotion apply plan")?;
    writeln!(file, ":PROPERTIES:")?;
    writeln!(file, ":WENDAO_KIND: ontology_promotion_apply_plan")?;
    writeln!(file, ":SOURCE_MUTATION_ALLOWED: false")?;
    writeln!(file, ":ONTOLOGY_TRUTH: false")?;
    writeln!(file, ":END:")?;
    writeln!(file)?;
    writeln!(
        file,
        "This plan summarizes explicit review approvals. It does not mutate source ontology RDF."
    )?;
    writeln!(file)?;
    writeln!(file, "| field | value |")?;
    writeln!(file, "|-|-|")?;
    writeln!(
        file,
        "| promotion_review_row_count | {} |",
        report.promotion_review_row_count
    )?;
    writeln!(
        file,
        "| pending_review_count | {} |",
        report.pending_review_count
    )?;
    writeln!(file, "| approved_count | {} |", report.approved_count)?;
    writeln!(file, "| rejected_count | {} |", report.rejected_count)?;
    writeln!(
        file,
        "| needs_evidence_count | {} |",
        report.needs_evidence_count
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

fn write_json(path: &Path, report: &EpistemeOntologyPromotionApplyPlanReport) -> Result<()> {
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

fn escape_tsv(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn unescape_tsv(value: &str) -> String {
    let mut output = String::new();
    let mut chars = value.chars();
    while let Some(character) = chars.next() {
        if character != '\\' {
            output.push(character);
            continue;
        }
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
    }
    output
}
