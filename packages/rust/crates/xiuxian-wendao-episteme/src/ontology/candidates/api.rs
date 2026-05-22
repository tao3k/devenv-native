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
