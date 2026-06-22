use std::collections::HashMap;

use serde::Deserialize;

pub(super) const RDF_DRAFT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_rdf_draft_export.v1";
pub(super) const OBJECTS_TSV: &str = "candidate_objects.tsv";
pub(super) const RELATIONS_TSV: &str = "candidate_relations.tsv";
pub(super) const EVIDENCE_TSV: &str = "candidate_evidence.tsv";
pub(super) const REVIEW_ORG: &str = "candidate_review.org";
pub(super) const QUALITY_REPORT_JSON: &str = "quality_report.json";
pub(super) const RDF_DRAFT_TTL: &str = "rdf_draft.ttl";
pub(super) const PROMOTION_PROPOSAL_ORG: &str = "promotion_proposal.org";
pub(super) const PROMOTION_PROPOSAL_JSON: &str = "promotion_proposal.json";
pub(super) const PROPOSAL_STATUS: &str = "draft_pending_review";
pub(super) const RAW_TO_RDF_PROMOTION_ALLOWED: bool = false;
pub(super) const ONTOLOGY_TRUTH: bool = false;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct QualityReport {
    pub(super) candidate_object_count: usize,
    pub(super) candidate_relation_count: usize,
    pub(super) candidate_evidence_count: usize,
    pub(super) review_row_count: usize,
    pub(super) review_gate_passed: bool,
}

#[derive(Debug, Clone)]
pub(super) struct CandidateObjectRecord {
    pub(super) candidate_id: String,
    pub(super) candidate_kind: String,
    pub(super) label: String,
    pub(super) suggested_term_key: String,
    pub(super) source_file_id: String,
    pub(super) source_queue_id: String,
    pub(super) source_path: String,
    pub(super) category: String,
    pub(super) language: String,
    pub(super) extraction_route: String,
    pub(super) extraction_run_id: String,
    pub(super) evidence_sha256: String,
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
pub(super) struct ReviewRecord {
    pub(super) record_id: String,
    pub(super) record_kind: String,
    pub(super) review_decision: String,
    pub(super) quality_score: usize,
    pub(super) evidence_strength: String,
    pub(super) issue_codes: String,
    pub(super) promotion_precondition_met: bool,
    pub(super) suggested_term_key: String,
    pub(super) label: String,
}

#[derive(Debug)]
pub(super) struct DraftInputs {
    pub(super) objects: Vec<CandidateObjectRecord>,
    pub(super) relations: Vec<CandidateRelationRecord>,
    pub(super) evidence: Vec<CandidateEvidenceRecord>,
    pub(super) reviews_by_id: HashMap<String, ReviewRecord>,
    pub(super) quality: QualityReport,
}

#[derive(Debug)]
pub(super) struct DraftRender {
    pub(super) ttl: String,
    pub(super) resource_count: usize,
    pub(super) statement_count: usize,
}

pub(super) struct RenderedResource {
    pub(super) text: String,
    pub(super) statement_count: usize,
}

pub(super) struct TsvTable {
    pub(super) rows: Vec<HashMap<String, String>>,
}
