//! Public candidate-generation API types.

use std::path::PathBuf;

use serde::Serialize;

/// Request for writing review-gated ontology candidate artifacts.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeOntologyCandidateGenerationRequest {
    /// Episteme repository root.
    pub episteme_root: PathBuf,
    /// Safe ASCII run id.
    pub run_id: String,
    /// Extraction run root containing optional cache evidence runs.
    pub extraction_run_root: PathBuf,
    /// Optional extraction run ids whose cache outputs should seed evidence rows.
    pub extraction_run_ids: Vec<String>,
}

impl EpistemeOntologyCandidateGenerationRequest {
    /// Create an ontology candidate generation request.
    #[must_use]
    pub fn new(
        episteme_root: impl Into<PathBuf>,
        run_id: impl Into<String>,
        extraction_run_root: impl Into<PathBuf>,
    ) -> Self {
        Self {
            episteme_root: episteme_root.into(),
            run_id: run_id.into(),
            extraction_run_root: extraction_run_root.into(),
            extraction_run_ids: Vec::new(),
        }
    }

    /// Attach extraction run ids used for cache-evidence enrichment.
    #[must_use]
    pub fn with_extraction_run_ids(mut self, run_ids: impl IntoIterator<Item = String>) -> Self {
        self.extraction_run_ids = run_ids.into_iter().collect();
        self
    }
}

/// Request for summarizing the typed candidate Arrow/Parquet read model.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeOntologyCandidateReadModelSummaryRequest {
    /// Candidate object Parquet read-model path.
    pub objects: PathBuf,
    /// Candidate relation Parquet read-model path.
    pub relations: PathBuf,
    /// Candidate evidence Parquet read-model path.
    pub evidence: PathBuf,
}

impl EpistemeOntologyCandidateReadModelSummaryRequest {
    /// Create a read-model summary request from explicit Parquet paths.
    #[must_use]
    pub fn new(
        candidate_objects_parquet: impl Into<PathBuf>,
        candidate_relations_parquet: impl Into<PathBuf>,
        candidate_evidence_parquet: impl Into<PathBuf>,
    ) -> Self {
        Self {
            objects: candidate_objects_parquet.into(),
            relations: candidate_relations_parquet.into(),
            evidence: candidate_evidence_parquet.into(),
        }
    }

    /// Create a read-model summary request from a candidate generation report.
    #[must_use]
    pub fn from_generation_report(report: &EpistemeOntologyCandidateGenerationReport) -> Self {
        Self {
            objects: report.candidate_objects_parquet.clone(),
            relations: report.candidate_relations_parquet.clone(),
            evidence: report.candidate_evidence_parquet.clone(),
        }
    }
}

/// Report emitted after candidate generation.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyCandidateGenerationReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Safe ASCII run id.
    pub run_id: String,
    /// Concrete run directory.
    pub run_dir: PathBuf,
    /// Candidate object TSV path.
    pub candidate_objects_tsv: PathBuf,
    /// Candidate relation TSV path.
    pub candidate_relations_tsv: PathBuf,
    /// Candidate evidence TSV path.
    pub candidate_evidence_tsv: PathBuf,
    /// Candidate object Parquet read-model path.
    pub candidate_objects_parquet: PathBuf,
    /// Candidate relation Parquet read-model path.
    pub candidate_relations_parquet: PathBuf,
    /// Candidate evidence Parquet read-model path.
    pub candidate_evidence_parquet: PathBuf,
    /// Review ledger Org path.
    pub review_ledger_org: PathBuf,
    /// Receipt JSON path.
    pub receipt_json: PathBuf,
    /// Selected source-contract domain id.
    pub domain: String,
    /// Source files represented as source-artifact candidates.
    pub source_file_count: usize,
    /// Mapping term candidates read from the mapping ledger.
    pub mapping_term_count: usize,
    /// Extraction cache rows represented as evidence candidates.
    pub extraction_evidence_count: usize,
    /// Number of candidate object rows written.
    pub candidate_object_count: usize,
    /// Number of candidate relation rows written.
    pub candidate_relation_count: usize,
    /// Number of candidate evidence rows written.
    pub candidate_evidence_count: usize,
    /// Whether raw cache rows may be promoted directly to RDF.
    pub raw_to_rdf_promotion_allowed: bool,
    /// Whether generated rows are ontology truth.
    pub ontology_truth: bool,
}

/// Missing relation endpoint found while reading the candidate read model.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyCandidateReadModelMissingEndpoint {
    /// Relation candidate id whose endpoint is missing.
    pub relation_candidate_id: String,
    /// Missing endpoint role: `source` or `target`.
    pub endpoint_role: String,
    /// Candidate id referenced by the relation but absent from object rows.
    pub endpoint_candidate_id: String,
}

/// Summary and quality gate for the typed candidate read-model tables.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyCandidateReadModelSummaryReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Candidate object Parquet read-model path.
    pub candidate_objects_parquet: PathBuf,
    /// Candidate relation Parquet read-model path.
    pub candidate_relations_parquet: PathBuf,
    /// Candidate evidence Parquet read-model path.
    pub candidate_evidence_parquet: PathBuf,
    /// Number of object rows in the read model.
    pub candidate_object_count: usize,
    /// Number of relation rows in the read model.
    pub candidate_relation_count: usize,
    /// Number of evidence rows in the read model.
    pub candidate_evidence_count: usize,
    /// Rows whose review status is not review-required.
    pub review_status_violation_count: usize,
    /// Rows whose promotion status is not blocked pending review.
    pub promotion_status_violation_count: usize,
    /// Rows that incorrectly claim ontology truth.
    pub ontology_truth_violation_count: usize,
    /// Object rows that incorrectly allow raw-to-RDF promotion.
    pub raw_to_rdf_promotion_violation_count: usize,
    /// Relation endpoints absent from candidate object rows.
    pub missing_relation_endpoint_count: usize,
    /// Concrete missing relation endpoints.
    pub missing_relation_endpoints: Vec<EpistemeOntologyCandidateReadModelMissingEndpoint>,
    /// Whether the read-model quality gate passed.
    pub read_model_gate_passed: bool,
}
