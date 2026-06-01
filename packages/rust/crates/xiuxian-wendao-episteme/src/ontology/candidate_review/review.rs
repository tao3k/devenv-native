//! Candidate review scoring and deterministic gate decisions.

use std::collections::{BTreeSet, HashSet};

use super::model::{
    CandidateEvidenceRecord, CandidateObjectRecord, CandidateRelationRecord, ReviewMetrics,
    ReviewRow,
};

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

pub(super) fn build_review_rows(
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
