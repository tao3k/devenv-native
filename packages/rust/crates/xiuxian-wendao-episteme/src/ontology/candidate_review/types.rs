//! Candidate review public contracts and file-name constants.

use std::path::{Path, PathBuf};

use serde::Serialize;

pub(super) const REVIEW_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_candidate_review.v1";
pub(super) const OBJECTS_TSV: &str = "candidate_objects.tsv";
pub(super) const RELATIONS_TSV: &str = "candidate_relations.tsv";
pub(super) const EVIDENCE_TSV: &str = "candidate_evidence.tsv";
pub(super) const REVIEW_TSV: &str = "candidate_review.tsv";
pub(super) const REVIEW_ORG: &str = "candidate_review.org";
pub(super) const QUALITY_REPORT_JSON: &str = "quality_report.json";
pub(super) const REVIEW_COLUMNS: [&str; 12] = [
    "record_id",
    "record_kind",
    "review_decision",
    "quality_score",
    "evidence_strength",
    "issue_codes",
    "promotion_precondition_met",
    "source_file_id",
    "source_queue_id",
    "extraction_run_id",
    "suggested_term_key",
    "label",
];

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
    /// Authoritative candidate review Org ledger path.
    pub candidate_review_org: PathBuf,
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
