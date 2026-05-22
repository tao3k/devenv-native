//! Deterministic review gate for generated ontology candidate artifacts.

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Serialize;

const REVIEW_SCHEMA_VERSION: &str = "xiuxian_wendao.episteme_ontology_candidate_review.v1";
const OBJECTS_TSV: &str = "candidate_objects.tsv";
const RELATIONS_TSV: &str = "candidate_relations.tsv";
const EVIDENCE_TSV: &str = "candidate_evidence.tsv";
const REVIEW_TSV: &str = "candidate_review.tsv";
const QUALITY_REPORT_JSON: &str = "quality_report.json";

/// Request for reviewing generated ontology candidate artifacts.
#[derive(Debug, Clone)]
pub struct EpistemeOntologyCandidateReviewRequest {
    run_dir: PathBuf,
}

impl EpistemeOntologyCandidateReviewRequest {
    /// Create a review request from an ontology-generation run directory.
    #[must_use]
    pub fn new(run_dir: impl Into<PathBuf>) -> Self {
        Self {
            run_dir: run_dir.into(),
        }
    }

    /// Ontology-generation run directory that contains generated candidate TSVs.
    #[must_use]
    pub fn run_dir(&self) -> &Path {
        self.run_dir.as_path()
    }
}

/// Report emitted after ontology candidate review.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyCandidateReviewReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Reviewed ontology-generation run directory.
    pub run_dir: PathBuf,
    /// Generated candidate review TSV path.
    pub candidate_review_tsv: PathBuf,
    /// Generated quality report JSON path.
    pub quality_report_json: PathBuf,
    /// Number of candidate object rows read.
    pub candidate_object_count: usize,
    /// Number of candidate relation rows read.
    pub candidate_relation_count: usize,
    /// Number of candidate evidence rows read.
    pub candidate_evidence_count: usize,
    /// Number of review rows written.
    pub review_row_count: usize,
    /// Number of duplicate candidate object ids.
    pub duplicate_candidate_id_count: usize,
    /// Number of relation rows with a missing source or target reference.
    pub missing_relation_reference_count: usize,
    /// Number of rows that attempted raw-to-RDF promotion.
    pub promotion_flag_violation_count: usize,
    /// Number of rows already marked as ontology truth.
    pub ontology_truth_violation_count: usize,
    /// Number of malformed rows or empty required fields.
    pub malformed_row_count: usize,
    /// Rows that meet the deterministic review precondition for later promotion review.
    pub promotion_precondition_met_count: usize,
    /// Rows blocked by invalid structure or unsafe flags.
    pub blocked_invalid_count: usize,
    /// Rows that are valid but need stronger evidence before promotion review.
    pub needs_evidence_count: usize,
    /// Whether the review gate passed without invalid rows.
    pub review_gate_passed: bool,
}

#[derive(Debug, Clone)]
struct CandidateObjectRecord {
    candidate_id: String,
    candidate_kind: String,
    label: String,
    suggested_term_key: String,
    source_file_id: String,
    source_queue_id: String,
    extraction_run_id: String,
    evidence_sha256: String,
    text_char_count: usize,
    raw_to_rdf_promotion_allowed: bool,
    ontology_truth: bool,
}

#[derive(Debug, Clone)]
struct CandidateRelationRecord {
    candidate_id: String,
    relation_kind: String,
    source_candidate_id: String,
    target_candidate_id: String,
    source_file_id: String,
    source_queue_id: String,
    extraction_run_id: String,
    evidence_sha256: String,
    ontology_truth: bool,
}

#[derive(Debug, Clone)]
struct CandidateEvidenceRecord {
    evidence_id: String,
    evidence_kind: String,
    source_file_id: String,
    source_queue_id: String,
    extraction_run_id: String,
    evidence_sha256: String,
    text_char_count: usize,
    ontology_truth: bool,
}

#[derive(Debug, Clone)]
struct ReviewRow {
    record_id: String,
    record_kind: String,
    review_decision: &'static str,
    quality_score: u8,
    evidence_strength: &'static str,
    issue_codes: Vec<&'static str>,
    promotion_precondition_met: bool,
    source_file_id: String,
    source_queue_id: String,
    extraction_run_id: String,
    suggested_term_key: String,
    label: String,
}

#[derive(Debug)]
struct ReviewRowInput {
    record_id: String,
    record_kind: String,
    label: String,
    suggested_term_key: String,
    source_file_id: String,
    source_queue_id: String,
    extraction_run_id: String,
    evidence_strength: &'static str,
    issue_codes: Vec<&'static str>,
    quality_score: u8,
}

#[derive(Debug, Default)]
struct ReviewMetrics {
    duplicate_candidate_ids: BTreeSet<String>,
    missing_relation_reference_count: usize,
    promotion_flag_violation_count: usize,
    ontology_truth_violation_count: usize,
    malformed_row_count: usize,
    promotion_precondition_met_count: usize,
    blocked_invalid_count: usize,
    needs_evidence_count: usize,
}

/// Review generated candidate TSVs and write deterministic quality artifacts.
///
/// # Errors
///
/// Returns an error when required TSV files are missing, malformed, or cannot
/// be read/written.
pub fn review_episteme_ontology_candidates(
    request: &EpistemeOntologyCandidateReviewRequest,
) -> Result<EpistemeOntologyCandidateReviewReport> {
    let run_dir = request.run_dir();
    let objects = read_candidate_objects(run_dir.join(OBJECTS_TSV).as_path())?;
    let relations = read_candidate_relations(run_dir.join(RELATIONS_TSV).as_path())?;
    let evidence = read_candidate_evidence(run_dir.join(EVIDENCE_TSV).as_path())?;
    let (review_rows, metrics) = build_review_rows(&objects, &relations, &evidence);
    let review_tsv = run_dir.join(REVIEW_TSV);
    let quality_report_json = run_dir.join(QUALITY_REPORT_JSON);
    write_review_tsv(review_tsv.as_path(), &review_rows)?;
    let report = EpistemeOntologyCandidateReviewReport {
        schema_version: REVIEW_SCHEMA_VERSION,
        run_dir: run_dir.to_path_buf(),
        candidate_review_tsv: review_tsv,
        quality_report_json,
        candidate_object_count: objects.len(),
        candidate_relation_count: relations.len(),
        candidate_evidence_count: evidence.len(),
        review_row_count: review_rows.len(),
        duplicate_candidate_id_count: metrics.duplicate_candidate_ids.len(),
        missing_relation_reference_count: metrics.missing_relation_reference_count,
        promotion_flag_violation_count: metrics.promotion_flag_violation_count,
        ontology_truth_violation_count: metrics.ontology_truth_violation_count,
        malformed_row_count: metrics.malformed_row_count,
        promotion_precondition_met_count: metrics.promotion_precondition_met_count,
        blocked_invalid_count: metrics.blocked_invalid_count,
        needs_evidence_count: metrics.needs_evidence_count,
        review_gate_passed: metrics.blocked_invalid_count == 0,
    };
    write_json(report.quality_report_json.as_path(), &report)?;
    Ok(report)
}

fn build_review_rows(
    objects: &[CandidateObjectRecord],
    relations: &[CandidateRelationRecord],
    evidence: &[CandidateEvidenceRecord],
) -> (Vec<ReviewRow>, ReviewMetrics) {
    let mut metrics = ReviewMetrics {
        duplicate_candidate_ids: collect_duplicate_candidate_ids(objects),
        ..ReviewMetrics::default()
    };
    let candidate_ids: HashSet<&str> = objects
        .iter()
        .map(|object| object.candidate_id.as_str())
        .collect();
    let rows = collect_review_rows(objects, relations, evidence, &candidate_ids, &metrics);
    for row in &rows {
        accumulate_review_metrics(row, &mut metrics);
    }

    (rows, metrics)
}

fn collect_duplicate_candidate_ids(objects: &[CandidateObjectRecord]) -> BTreeSet<String> {
    let mut seen_ids = HashSet::new();
    objects
        .iter()
        .filter_map(|object| {
            if seen_ids.insert(object.candidate_id.clone()) {
                None
            } else {
                Some(object.candidate_id.clone())
            }
        })
        .collect()
}

fn collect_review_rows(
    objects: &[CandidateObjectRecord],
    relations: &[CandidateRelationRecord],
    evidence: &[CandidateEvidenceRecord],
    candidate_ids: &HashSet<&str>,
    metrics: &ReviewMetrics,
) -> Vec<ReviewRow> {
    objects
        .iter()
        .map(|object| review_object_row(object, &metrics.duplicate_candidate_ids))
        .chain(
            relations
                .iter()
                .map(|relation| review_relation_row(relation, candidate_ids)),
        )
        .chain(evidence.iter().map(review_evidence_row))
        .collect()
}

fn accumulate_review_metrics(row: &ReviewRow, metrics: &mut ReviewMetrics) {
    if row.issue_codes.contains(&"missing_relation_reference") {
        metrics.missing_relation_reference_count += 1;
    }
    if row.issue_codes.contains(&"promotion_flag_violation") {
        metrics.promotion_flag_violation_count += 1;
    }
    if row.issue_codes.contains(&"ontology_truth_violation") {
        metrics.ontology_truth_violation_count += 1;
    }
    if has_malformed_issue(row) {
        metrics.malformed_row_count += 1;
    }
    if row.promotion_precondition_met {
        metrics.promotion_precondition_met_count += 1;
    } else if row.review_decision == "blocked_invalid" {
        metrics.blocked_invalid_count += 1;
    } else if row.review_decision == "needs_evidence" {
        metrics.needs_evidence_count += 1;
    }
}

fn has_malformed_issue(row: &ReviewRow) -> bool {
    row.issue_codes.iter().any(|issue| {
        matches!(
            *issue,
            "empty_id"
                | "empty_kind"
                | "empty_label"
                | "duplicate_candidate_id"
                | "missing_relation_reference"
        )
    })
}

fn review_object_row(
    object: &CandidateObjectRecord,
    duplicate_ids: &BTreeSet<String>,
) -> ReviewRow {
    let mut issues = Vec::new();
    if object.candidate_id.trim().is_empty() {
        issues.push("empty_id");
    }
    if object.candidate_kind.trim().is_empty() {
        issues.push("empty_kind");
    }
    if object.label.trim().is_empty() {
        issues.push("empty_label");
    }
    if duplicate_ids.contains(&object.candidate_id) {
        issues.push("duplicate_candidate_id");
    }
    if object.raw_to_rdf_promotion_allowed {
        issues.push("promotion_flag_violation");
    }
    if object.ontology_truth {
        issues.push("ontology_truth_violation");
    }
    let evidence_strength = object_evidence_strength(object);
    review_row(ReviewRowInput {
        record_id: object.candidate_id.clone(),
        record_kind: object.candidate_kind.clone(),
        label: object.label.clone(),
        suggested_term_key: object.suggested_term_key.clone(),
        source_file_id: object.source_file_id.clone(),
        source_queue_id: object.source_queue_id.clone(),
        extraction_run_id: object.extraction_run_id.clone(),
        evidence_strength,
        issue_codes: issues,
        quality_score: score_object(object, evidence_strength),
    })
}

fn review_relation_row(
    relation: &CandidateRelationRecord,
    candidate_ids: &HashSet<&str>,
) -> ReviewRow {
    let mut issues = Vec::new();
    if relation.candidate_id.trim().is_empty() {
        issues.push("empty_id");
    }
    if relation.relation_kind.trim().is_empty() {
        issues.push("empty_kind");
    }
    if !candidate_ids.contains(relation.source_candidate_id.as_str())
        || !candidate_ids.contains(relation.target_candidate_id.as_str())
    {
        issues.push("missing_relation_reference");
    }
    if relation.ontology_truth {
        issues.push("ontology_truth_violation");
    }
    let evidence_strength = if relation.evidence_sha256.trim().is_empty() {
        "none"
    } else {
        "hash_provenance"
    };
    review_row(ReviewRowInput {
        record_id: relation.candidate_id.clone(),
        record_kind: relation.relation_kind.clone(),
        label: String::new(),
        suggested_term_key: String::new(),
        source_file_id: relation.source_file_id.clone(),
        source_queue_id: relation.source_queue_id.clone(),
        extraction_run_id: relation.extraction_run_id.clone(),
        evidence_strength,
        issue_codes: issues,
        quality_score: score_relation(relation, evidence_strength),
    })
}

fn review_evidence_row(evidence: &CandidateEvidenceRecord) -> ReviewRow {
    let mut issues = Vec::new();
    if evidence.evidence_id.trim().is_empty() {
        issues.push("empty_id");
    }
    if evidence.evidence_kind.trim().is_empty() {
        issues.push("empty_kind");
    }
    if evidence.ontology_truth {
        issues.push("ontology_truth_violation");
    }
    let evidence_strength = if evidence.text_char_count > 0 {
        "extracted_text_hash"
    } else if evidence.evidence_sha256.trim().is_empty() {
        "none"
    } else {
        "hash_provenance"
    };
    review_row(ReviewRowInput {
        record_id: evidence.evidence_id.clone(),
        record_kind: evidence.evidence_kind.clone(),
        label: String::new(),
        suggested_term_key: String::new(),
        source_file_id: evidence.source_file_id.clone(),
        source_queue_id: evidence.source_queue_id.clone(),
        extraction_run_id: evidence.extraction_run_id.clone(),
        evidence_strength,
        issue_codes: issues,
        quality_score: score_evidence(evidence, evidence_strength),
    })
}

fn review_row(input: ReviewRowInput) -> ReviewRow {
    let ReviewRowInput {
        record_id,
        record_kind,
        label,
        suggested_term_key,
        source_file_id,
        source_queue_id,
        extraction_run_id,
        evidence_strength,
        issue_codes,
        quality_score,
    } = input;
    let review_decision = if issue_codes.iter().any(|issue| {
        matches!(
            *issue,
            "empty_id"
                | "empty_kind"
                | "empty_label"
                | "duplicate_candidate_id"
                | "missing_relation_reference"
                | "promotion_flag_violation"
                | "ontology_truth_violation"
        )
    }) {
        "blocked_invalid"
    } else if evidence_strength == "none" {
        "needs_evidence"
    } else {
        "ready_for_review"
    };
    ReviewRow {
        record_id,
        record_kind,
        review_decision,
        quality_score,
        evidence_strength,
        issue_codes,
        promotion_precondition_met: review_decision == "ready_for_review",
        source_file_id,
        source_queue_id,
        extraction_run_id,
        suggested_term_key,
        label,
    }
}

fn object_evidence_strength(object: &CandidateObjectRecord) -> &'static str {
    if object.text_char_count > 0 {
        "extracted_text_hash"
    } else if !object.extraction_run_id.trim().is_empty() {
        "cache_provenance"
    } else if !object.source_file_id.trim().is_empty() {
        "source_metadata"
    } else if !object.suggested_term_key.trim().is_empty() {
        "mapping_ledger"
    } else {
        "none"
    }
}

fn score_object(object: &CandidateObjectRecord, evidence_strength: &str) -> u8 {
    let mut score = 30;
    if !object.label.trim().is_empty() {
        score += 15;
    }
    if !object.suggested_term_key.trim().is_empty() {
        score += 15;
    }
    if !object.source_file_id.trim().is_empty() {
        score += 15;
    }
    if !object.evidence_sha256.trim().is_empty() {
        score += 5;
    }
    score + evidence_bonus(evidence_strength)
}

fn score_relation(relation: &CandidateRelationRecord, evidence_strength: &str) -> u8 {
    let mut score = 35;
    if !relation.source_candidate_id.trim().is_empty()
        && !relation.target_candidate_id.trim().is_empty()
    {
        score += 20;
    }
    if !relation.source_file_id.trim().is_empty() {
        score += 10;
    }
    score + evidence_bonus(evidence_strength)
}

fn score_evidence(evidence: &CandidateEvidenceRecord, evidence_strength: &str) -> u8 {
    let mut score = 40;
    if !evidence.source_file_id.trim().is_empty() {
        score += 15;
    }
    if !evidence.extraction_run_id.trim().is_empty() {
        score += 10;
    }
    score + evidence_bonus(evidence_strength)
}

fn evidence_bonus(evidence_strength: &str) -> u8 {
    match evidence_strength {
        "extracted_text_hash" => 30,
        "cache_provenance" => 25,
        "source_metadata" | "hash_provenance" => 20,
        "mapping_ledger" => 15,
        _ => 0,
    }
}

fn read_candidate_objects(path: &Path) -> Result<Vec<CandidateObjectRecord>> {
    let table = read_tsv(path)?;
    table
        .rows
        .iter()
        .map(|row| {
            Ok(CandidateObjectRecord {
                candidate_id: required(row, "candidate_id", path)?,
                candidate_kind: required(row, "candidate_kind", path)?,
                label: required(row, "label", path)?,
                suggested_term_key: optional(row, "suggested_term_key"),
                source_file_id: optional(row, "source_file_id"),
                source_queue_id: optional(row, "source_queue_id"),
                extraction_run_id: optional(row, "extraction_run_id"),
                evidence_sha256: optional(row, "evidence_sha256"),
                text_char_count: parse_usize(row, "text_char_count", path)?,
                raw_to_rdf_promotion_allowed: parse_bool(
                    row,
                    "raw_to_rdf_promotion_allowed",
                    path,
                )?,
                ontology_truth: parse_bool(row, "ontology_truth", path)?,
            })
        })
        .collect()
}

fn read_candidate_relations(path: &Path) -> Result<Vec<CandidateRelationRecord>> {
    let table = read_tsv(path)?;
    table
        .rows
        .iter()
        .map(|row| {
            Ok(CandidateRelationRecord {
                candidate_id: required(row, "candidate_id", path)?,
                relation_kind: required(row, "relation_kind", path)?,
                source_candidate_id: required(row, "source_candidate_id", path)?,
                target_candidate_id: required(row, "target_candidate_id", path)?,
                source_file_id: optional(row, "source_file_id"),
                source_queue_id: optional(row, "source_queue_id"),
                extraction_run_id: optional(row, "extraction_run_id"),
                evidence_sha256: optional(row, "evidence_sha256"),
                ontology_truth: parse_bool(row, "ontology_truth", path)?,
            })
        })
        .collect()
}

fn read_candidate_evidence(path: &Path) -> Result<Vec<CandidateEvidenceRecord>> {
    let table = read_tsv(path)?;
    table
        .rows
        .iter()
        .map(|row| {
            Ok(CandidateEvidenceRecord {
                evidence_id: required(row, "evidence_id", path)?,
                evidence_kind: required(row, "evidence_kind", path)?,
                source_file_id: optional(row, "source_file_id"),
                source_queue_id: optional(row, "source_queue_id"),
                extraction_run_id: optional(row, "extraction_run_id"),
                evidence_sha256: optional(row, "evidence_sha256"),
                text_char_count: parse_usize(row, "text_char_count", path)?,
                ontology_truth: parse_bool(row, "ontology_truth", path)?,
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

fn write_review_tsv(path: &Path, rows: &[ReviewRow]) -> Result<()> {
    let mut file = create_file(path)?;
    writeln!(
        file,
        "record_id\trecord_kind\treview_decision\tquality_score\tevidence_strength\tissue_codes\tpromotion_precondition_met\tsource_file_id\tsource_queue_id\textraction_run_id\tsuggested_term_key\tlabel"
    )?;
    for row in rows {
        writeln!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            escape_tsv(&row.record_id),
            escape_tsv(&row.record_kind),
            row.review_decision,
            row.quality_score,
            row.evidence_strength,
            escape_tsv(&row.issue_codes.join(",")),
            row.promotion_precondition_met,
            escape_tsv(&row.source_file_id),
            escape_tsv(&row.source_queue_id),
            escape_tsv(&row.extraction_run_id),
            escape_tsv(&row.suggested_term_key),
            escape_tsv(&row.label)
        )?;
    }
    Ok(())
}

fn write_json(path: &Path, report: &EpistemeOntologyCandidateReviewReport) -> Result<()> {
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
