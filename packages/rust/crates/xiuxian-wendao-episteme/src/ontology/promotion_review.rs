//! Promotion review packet generation for ontology RDF draft runs.

use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const PROMOTION_REVIEW_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_promotion_review_packet.v1";
const RELATIONS_TSV: &str = "candidate_relations.tsv";
const REVIEW_TSV: &str = "candidate_review.tsv";
const RDF_DRAFT_TTL: &str = "rdf_draft.ttl";
const PROMOTION_PROPOSAL_JSON: &str = "promotion_proposal.json";
const PROMOTION_REVIEW_TSV: &str = "promotion_review.tsv";
const PROMOTION_REVIEW_ORG: &str = "promotion_review.org";
const PROMOTION_REVIEW_JSON: &str = "promotion_review.json";
const PROMOTION_DECISION_PENDING: &str = "pending_review";
const SOURCE_MUTATION_ALLOWED: bool = false;
const ONTOLOGY_TRUTH: bool = false;

/// Request for writing a promotion review packet from a clean RDF draft run.
#[derive(Debug, Clone)]
pub struct EpistemeOntologyPromotionReviewPacketRequest {
    run_dir: PathBuf,
}

impl EpistemeOntologyPromotionReviewPacketRequest {
    /// Create a promotion review packet request from an ontology-generation run directory.
    #[must_use]
    pub fn new(run_dir: impl Into<PathBuf>) -> Self {
        Self {
            run_dir: run_dir.into(),
        }
    }

    /// Ontology-generation run directory that contains a clean draft proposal.
    #[must_use]
    pub fn run_dir(&self) -> &Path {
        self.run_dir.as_path()
    }
}

/// Report emitted after promotion review packet generation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyPromotionReviewPacketReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Reviewed ontology-generation run directory.
    pub run_dir: PathBuf,
    /// Generated promotion review TSV path.
    pub promotion_review_tsv: PathBuf,
    /// Generated promotion review Org path.
    pub promotion_review_org: PathBuf,
    /// Generated promotion review JSON path.
    pub promotion_review_json: PathBuf,
    /// Number of review rows read from `candidate_review.tsv`.
    pub review_row_count: usize,
    /// Number of promotion review packet rows written.
    pub promotion_review_row_count: usize,
    /// Number of rows that are pending review.
    pub pending_review_count: usize,
    /// Whether the upstream review gate passed.
    pub review_gate_passed: bool,
    /// Whether source mutation is authorized by this generated packet.
    pub source_mutation_allowed: bool,
    /// Whether generated rows are ontology truth.
    pub ontology_truth: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PromotionProposalReport {
    rdf_draft_ttl: PathBuf,
    review_row_count: usize,
    review_gate_passed: bool,
    raw_to_rdf_promotion_allowed: bool,
    ontology_truth: bool,
}

#[derive(Debug, Clone)]
struct CandidateRelationRecord {
    id: String,
    source_id: String,
    target_id: String,
}

#[derive(Debug, Clone)]
struct CandidateReviewRecord {
    record_id: String,
    record_kind: String,
    review_decision: String,
    quality_score: usize,
    evidence_strength: String,
    issue_codes: String,
    promotion_precondition_met: bool,
    source_file_id: String,
    source_queue_id: String,
    extraction_run_id: String,
    suggested_term_key: String,
    label: String,
}

#[derive(Debug, Clone)]
struct PromotionReviewRow {
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
    promotion_decision: &'static str,
    source_mutation_allowed: bool,
    ontology_truth: bool,
    reviewer_id: String,
    reviewer_note: String,
}

struct PromotionReviewInputs {
    proposal: PromotionProposalReport,
    relation_context: HashMap<String, CandidateRelationRecord>,
    review_rows: Vec<CandidateReviewRecord>,
}

struct PromotionReviewOutputPaths {
    tsv: PathBuf,
    org: PathBuf,
    json: PathBuf,
}

/// Write a promotion review packet from a clean RDF draft run.
///
/// # Errors
///
/// Returns an error when the draft proposal is missing, unsafe, inconsistent
/// with the candidate review table, or when packet artifacts cannot be written.
pub fn write_episteme_ontology_promotion_review_packet(
    request: &EpistemeOntologyPromotionReviewPacketRequest,
) -> Result<EpistemeOntologyPromotionReviewPacketReport> {
    let run_dir = request.run_dir();
    let inputs = read_promotion_review_inputs(run_dir)?;
    let packet_rows = build_promotion_review_rows(&inputs.review_rows, &inputs.relation_context);
    let output_paths = promotion_review_output_paths(run_dir);
    write_promotion_review_tsv(output_paths.tsv.as_path(), &packet_rows)?;

    let report = build_promotion_review_report(run_dir, &inputs, &output_paths, &packet_rows);
    write_promotion_review_outputs(&output_paths, &report)?;
    Ok(report)
}

fn read_promotion_review_inputs(run_dir: &Path) -> Result<PromotionReviewInputs> {
    let proposal = read_promotion_proposal(run_dir.join(PROMOTION_PROPOSAL_JSON).as_path())?;
    validate_promotion_proposal(run_dir, &proposal)?;
    let relation_context = read_candidate_relations(run_dir.join(RELATIONS_TSV).as_path())?;
    let review_rows = read_candidate_review(run_dir.join(REVIEW_TSV).as_path())?;
    ensure_review_row_count_matches(&proposal, review_rows.len())?;
    Ok(PromotionReviewInputs {
        proposal,
        relation_context,
        review_rows,
    })
}

fn ensure_review_row_count_matches(
    proposal: &PromotionProposalReport,
    review_row_count: usize,
) -> Result<()> {
    if proposal.review_row_count != review_row_count {
        anyhow::bail!(
            "promotion proposal review row count {} does not match candidate review row count {}",
            proposal.review_row_count,
            review_row_count
        );
    }
    Ok(())
}

fn promotion_review_output_paths(run_dir: &Path) -> PromotionReviewOutputPaths {
    PromotionReviewOutputPaths {
        tsv: run_dir.join(PROMOTION_REVIEW_TSV),
        org: run_dir.join(PROMOTION_REVIEW_ORG),
        json: run_dir.join(PROMOTION_REVIEW_JSON),
    }
}

fn build_promotion_review_report(
    run_dir: &Path,
    inputs: &PromotionReviewInputs,
    output_paths: &PromotionReviewOutputPaths,
    packet_rows: &[PromotionReviewRow],
) -> EpistemeOntologyPromotionReviewPacketReport {
    EpistemeOntologyPromotionReviewPacketReport {
        schema_version: PROMOTION_REVIEW_SCHEMA_VERSION,
        run_dir: run_dir.to_path_buf(),
        promotion_review_tsv: output_paths.tsv.clone(),
        promotion_review_org: output_paths.org.clone(),
        promotion_review_json: output_paths.json.clone(),
        review_row_count: inputs.review_rows.len(),
        promotion_review_row_count: packet_rows.len(),
        pending_review_count: packet_rows.len(),
        review_gate_passed: inputs.proposal.review_gate_passed,
        source_mutation_allowed: SOURCE_MUTATION_ALLOWED,
        ontology_truth: ONTOLOGY_TRUTH,
    }
}

fn write_promotion_review_outputs(
    output_paths: &PromotionReviewOutputPaths,
    report: &EpistemeOntologyPromotionReviewPacketReport,
) -> Result<()> {
    write_promotion_review_org(output_paths.org.as_path(), report)?;
    write_json(output_paths.json.as_path(), report)
}

fn validate_promotion_proposal(run_dir: &Path, proposal: &PromotionProposalReport) -> Result<()> {
    if !proposal.review_gate_passed {
        anyhow::bail!("promotion review packet requires `reviewGatePassed=true`");
    }
    if proposal.raw_to_rdf_promotion_allowed {
        anyhow::bail!("promotion review packet requires raw-to-RDF promotion to be disabled");
    }
    if proposal.ontology_truth {
        anyhow::bail!("promotion review packet requires `ontologyTruth=false`");
    }
    let draft_path = if proposal.rdf_draft_ttl.is_absolute() {
        proposal.rdf_draft_ttl.clone()
    } else {
        run_dir.join(&proposal.rdf_draft_ttl)
    };
    if !draft_path.is_file() && !run_dir.join(RDF_DRAFT_TTL).is_file() {
        anyhow::bail!("promotion review packet requires `{RDF_DRAFT_TTL}`");
    }
    Ok(())
}

fn build_promotion_review_rows(
    review_rows: &[CandidateReviewRecord],
    relation_context: &HashMap<String, CandidateRelationRecord>,
) -> Vec<PromotionReviewRow> {
    review_rows
        .iter()
        .map(|review| {
            let relation = relation_context.get(review.record_id.as_str());
            PromotionReviewRow {
                record_id: review.record_id.clone(),
                record_kind: review.record_kind.clone(),
                label: review.label.clone(),
                suggested_term_key: review.suggested_term_key.clone(),
                review_decision: review.review_decision.clone(),
                quality_score: review.quality_score,
                evidence_strength: review.evidence_strength.clone(),
                issue_codes: review.issue_codes.clone(),
                promotion_precondition_met: review.promotion_precondition_met,
                source_file_id: review.source_file_id.clone(),
                source_queue_id: review.source_queue_id.clone(),
                extraction_run_id: review.extraction_run_id.clone(),
                relation_source_candidate_id: relation
                    .map(|record| record.source_id.clone())
                    .unwrap_or_default(),
                relation_target_candidate_id: relation
                    .map(|record| record.target_id.clone())
                    .unwrap_or_default(),
                promotion_decision: PROMOTION_DECISION_PENDING,
                source_mutation_allowed: SOURCE_MUTATION_ALLOWED,
                ontology_truth: ONTOLOGY_TRUTH,
                reviewer_id: String::new(),
                reviewer_note: String::new(),
            }
        })
        .collect()
}

fn read_promotion_proposal(path: &Path) -> Result<PromotionProposalReport> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    serde_json::from_str(raw.as_str())
        .with_context(|| format!("failed to parse `{}`", path.display()))
}

fn read_candidate_relations(path: &Path) -> Result<HashMap<String, CandidateRelationRecord>> {
    let table = read_tsv(path)?;
    table
        .rows
        .iter()
        .map(|row| {
            let record = CandidateRelationRecord {
                id: required(row, "candidate_id", path)?,
                source_id: required(row, "source_candidate_id", path)?,
                target_id: required(row, "target_candidate_id", path)?,
            };
            Ok((record.id.clone(), record))
        })
        .collect()
}

fn read_candidate_review(path: &Path) -> Result<Vec<CandidateReviewRecord>> {
    let table = read_tsv(path)?;
    table
        .rows
        .iter()
        .map(|row| {
            Ok(CandidateReviewRecord {
                record_id: required(row, "record_id", path)?,
                record_kind: required(row, "record_kind", path)?,
                review_decision: required(row, "review_decision", path)?,
                quality_score: parse_usize(row, "quality_score", path)?,
                evidence_strength: required(row, "evidence_strength", path)?,
                issue_codes: optional(row, "issue_codes"),
                promotion_precondition_met: parse_bool(row, "promotion_precondition_met", path)?,
                source_file_id: optional(row, "source_file_id"),
                source_queue_id: optional(row, "source_queue_id"),
                extraction_run_id: optional(row, "extraction_run_id"),
                suggested_term_key: optional(row, "suggested_term_key"),
                label: optional(row, "label"),
            })
        })
        .collect()
}

struct TsvTable {
    rows: Vec<HashMap<String, String>>,
}

fn read_tsv(path: &Path) -> Result<TsvTable> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read `{}`", path.display()))?;
    let mut lines = content.lines();
    let header_line = lines
        .next()
        .with_context(|| format!("candidate TSV `{}` is empty", path.display()))?;
    let headers: Vec<String> = header_line.split('\t').map(ToString::to_string).collect();
    if headers.is_empty() {
        anyhow::bail!("candidate TSV `{}` has no header", path.display());
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
) -> Result<HashMap<String, String>> {
    let values: Vec<&str> = line.split('\t').collect();
    if values.len() != headers.len() {
        anyhow::bail!(
            "candidate TSV `{}` row {} has {} columns, expected {}",
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

fn required(row: &HashMap<String, String>, name: &str, path: &Path) -> Result<String> {
    row.get(name)
        .cloned()
        .with_context(|| format!("candidate TSV `{}` missing `{name}` column", path.display()))
}

fn optional(row: &HashMap<String, String>, name: &str) -> String {
    row.get(name).cloned().unwrap_or_default()
}

fn parse_usize(row: &HashMap<String, String>, name: &str, path: &Path) -> Result<usize> {
    let value = required(row, name, path)?;
    value.parse::<usize>().with_context(|| {
        format!(
            "candidate TSV `{}` has invalid `{name}` value `{value}`",
            path.display()
        )
    })
}

fn parse_bool(row: &HashMap<String, String>, name: &str, path: &Path) -> Result<bool> {
    let value = required(row, name, path)?;
    match value.as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => anyhow::bail!(
            "candidate TSV `{}` has invalid `{name}` value `{value}`",
            path.display()
        ),
    }
}

fn write_promotion_review_tsv(path: &Path, rows: &[PromotionReviewRow]) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "record_id\trecord_kind\tlabel\tsuggested_term_key\treview_decision\tquality_score\tevidence_strength\tissue_codes\tpromotion_precondition_met\tsource_file_id\tsource_queue_id\textraction_run_id\trelation_source_candidate_id\trelation_target_candidate_id\tpromotion_decision\tsource_mutation_allowed\tontology_truth\treviewer_id\treviewer_note"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            escape_tsv(&row.record_id),
            escape_tsv(&row.record_kind),
            escape_tsv(&row.label),
            escape_tsv(&row.suggested_term_key),
            escape_tsv(&row.review_decision),
            row.quality_score,
            escape_tsv(&row.evidence_strength),
            escape_tsv(&row.issue_codes),
            row.promotion_precondition_met,
            escape_tsv(&row.source_file_id),
            escape_tsv(&row.source_queue_id),
            escape_tsv(&row.extraction_run_id),
            escape_tsv(&row.relation_source_candidate_id),
            escape_tsv(&row.relation_target_candidate_id),
            row.promotion_decision,
            row.source_mutation_allowed,
            row.ontology_truth,
            escape_tsv(&row.reviewer_id),
            escape_tsv(&row.reviewer_note)
        )?;
    }
    Ok(())
}

fn write_promotion_review_org(
    path: &Path,
    report: &EpistemeOntologyPromotionReviewPacketReport,
) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(file, "#+TITLE: Private Ontology Promotion Review Packet")?;
    writeln!(file)?;
    writeln!(file, "* Promotion review packet")?;
    writeln!(file, ":PROPERTIES:")?;
    writeln!(file, ":WENDAO_KIND: ontology_promotion_review_packet")?;
    writeln!(file, ":PROMOTION_STATE: pending_review")?;
    writeln!(file, ":SOURCE_MUTATION_ALLOWED: false")?;
    writeln!(file, ":ONTOLOGY_TRUTH: false")?;
    writeln!(file, ":END:")?;
    writeln!(file)?;
    writeln!(
        file,
        "This packet is a review input. It does not approve candidates or mutate source ontology RDF."
    )?;
    writeln!(file)?;
    writeln!(file, "| field | value |")?;
    writeln!(file, "|-|-|")?;
    writeln!(file, "| review_row_count | {} |", report.review_row_count)?;
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
    writeln!(
        file,
        "| review_gate_passed | {} |",
        report.review_gate_passed
    )?;
    writeln!(
        file,
        "| source_mutation_allowed | {} |",
        report.source_mutation_allowed
    )?;
    writeln!(file, "| ontology_truth | {} |", report.ontology_truth)?;
    Ok(())
}

fn write_json(path: &Path, report: &EpistemeOntologyPromotionReviewPacketReport) -> Result<()> {
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
