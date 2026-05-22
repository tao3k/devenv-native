//! Public RDF draft export API types.

use std::path::{Path, PathBuf};

use serde::Serialize;

/// Request for exporting review-gated RDF draft artifacts.
#[derive(Debug, Clone)]
pub struct EpistemeOntologyRdfDraftExportRequest {
    run_dir: PathBuf,
}

impl EpistemeOntologyRdfDraftExportRequest {
    /// Create an RDF draft export request from an ontology-generation run directory.
    #[must_use]
    pub fn new(run_dir: impl Into<PathBuf>) -> Self {
        Self {
            run_dir: run_dir.into(),
        }
    }

    /// Ontology-generation run directory that contains reviewed candidate artifacts.
    #[must_use]
    pub fn run_dir(&self) -> &Path {
        self.run_dir.as_path()
    }
}

/// Report emitted after RDF draft export.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyRdfDraftExportReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Reviewed ontology-generation run directory.
    pub run_dir: PathBuf,
    /// Generated RDF draft path.
    pub rdf_draft_ttl: PathBuf,
    /// Generated promotion proposal Org path.
    pub promotion_proposal_org: PathBuf,
    /// Generated promotion proposal JSON path.
    pub promotion_proposal_json: PathBuf,
    /// Number of candidate object rows read.
    pub candidate_object_count: usize,
    /// Number of candidate relation rows read.
    pub candidate_relation_count: usize,
    /// Number of candidate evidence rows read.
    pub candidate_evidence_count: usize,
    /// Number of candidate review rows read.
    pub review_row_count: usize,
    /// Number of RDF draft resources written.
    pub draft_resource_count: usize,
    /// Number of RDF draft statements written.
    pub draft_statement_count: usize,
    /// Whether the upstream review gate passed.
    pub review_gate_passed: bool,
    /// Whether raw generated rows may be promoted directly to RDF.
    pub raw_to_rdf_promotion_allowed: bool,
    /// Whether generated rows are ontology truth.
    pub ontology_truth: bool,
}
