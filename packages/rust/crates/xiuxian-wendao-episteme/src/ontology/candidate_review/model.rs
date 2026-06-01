//! Candidate review internal row models.

use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub(super) struct CandidateObjectRecord {
    pub(super) candidate_id: String,
    pub(super) candidate_kind: String,
    pub(super) label: String,
    pub(super) suggested_term_key: String,
    pub(super) source_file_id: String,
    pub(super) source_queue_id: String,
    pub(super) extraction_run_id: String,
    pub(super) evidence_sha256: String,
    pub(super) text_char_count: usize,
    pub(super) raw_to_rdf_promotion_allowed: bool,
    pub(super) ontology_truth: bool,
}

#[derive(Debug, Clone)]
pub(super) struct CandidateRelationRecord {
    pub(super) candidate_id: String,
    pub(super) relation_kind: String,
    pub(super) source_candidate_id: String,
    pub(super) target_candidate_id: String,
    pub(super) source_file_id: String,
    pub(super) source_queue_id: String,
    pub(super) extraction_run_id: String,
    pub(super) evidence_sha256: String,
    pub(super) ontology_truth: bool,
}

#[derive(Debug, Clone)]
pub(super) struct CandidateEvidenceRecord {
    pub(super) evidence_id: String,
    pub(super) evidence_kind: String,
    pub(super) source_file_id: String,
    pub(super) source_queue_id: String,
    pub(super) extraction_run_id: String,
    pub(super) evidence_sha256: String,
    pub(super) text_char_count: usize,
    pub(super) ontology_truth: bool,
}

#[derive(Debug, Clone)]
pub(super) struct ReviewRow {
    pub(super) record_id: String,
    pub(super) record_kind: String,
    pub(super) review_decision: &'static str,
    pub(super) quality_score: u8,
    pub(super) evidence_strength: &'static str,
    pub(super) issue_codes: Vec<&'static str>,
    pub(super) promotion_precondition_met: bool,
    pub(super) source_file_id: String,
    pub(super) source_queue_id: String,
    pub(super) extraction_run_id: String,
    pub(super) suggested_term_key: String,
    pub(super) label: String,
}

#[derive(Debug, Default)]
pub(super) struct ReviewMetrics {
    pub(super) duplicate_candidate_ids: BTreeSet<String>,
    pub(super) missing_relation_reference_count: usize,
    pub(super) promotion_flag_violation_count: usize,
    pub(super) ontology_truth_violation_count: usize,
    pub(super) malformed_row_count: usize,
    pub(super) promotion_precondition_met_count: usize,
    pub(super) blocked_invalid_count: usize,
    pub(super) needs_evidence_count: usize,
}
