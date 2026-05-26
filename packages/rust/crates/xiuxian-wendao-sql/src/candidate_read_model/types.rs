//! Public DTOs for candidate read-model `DuckDB` inspection.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::constants::{EVIDENCE_PARQUET, OBJECTS_PARQUET, RELATIONS_PARQUET};

/// Request for inspecting candidate Parquet read models through `DuckDB`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CandidateReadModelDuckDbInspectionRequest {
    /// Candidate objects Parquet path.
    pub objects: PathBuf,
    /// Candidate relations Parquet path.
    pub relations: PathBuf,
    /// Candidate evidence Parquet path.
    pub evidence: PathBuf,
}

impl CandidateReadModelDuckDbInspectionRequest {
    /// Create an inspection request from explicit Parquet paths.
    #[must_use]
    pub fn new(
        objects: impl Into<PathBuf>,
        relations: impl Into<PathBuf>,
        evidence: impl Into<PathBuf>,
    ) -> Self {
        Self {
            objects: objects.into(),
            relations: relations.into(),
            evidence: evidence.into(),
        }
    }

    /// Create an inspection request from the standard candidate run directory.
    #[must_use]
    pub fn from_candidate_run_dir(run_dir: impl Into<PathBuf>) -> Self {
        let run_dir = run_dir.into();
        Self {
            objects: run_dir.join(OBJECTS_PARQUET),
            relations: run_dir.join(RELATIONS_PARQUET),
            evidence: run_dir.join(EVIDENCE_PARQUET),
        }
    }
}

/// Count of rows grouped by one stable read-model kind.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateReadModelKindCount {
    /// Kind value from the read model.
    pub kind: CandidateReadModelKind,
    /// Number of rows for this kind.
    pub row_count: usize,
}

/// Stable kind value from a candidate read-model row.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CandidateReadModelKind(String);

impl CandidateReadModelKind {
    /// Borrow the stable kind token.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for CandidateReadModelKind {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for CandidateReadModelKind {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl PartialEq<&str> for CandidateReadModelKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// Missing endpoint detected in the relation read model.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateReadModelMissingEndpoint {
    /// Relation candidate id whose endpoint is missing.
    pub relation_candidate_id: String,
    /// Endpoint role: `source` or `target`.
    pub endpoint_role: String,
    /// Referenced candidate id missing from object rows.
    pub endpoint_candidate_id: String,
}

/// `DuckDB` inspection report for candidate Parquet read models.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateReadModelDuckDbInspectionReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// SQL execution engine.
    pub execution_engine: &'static str,
    /// How Parquet tables were registered.
    pub registration_strategy: &'static str,
    /// Candidate object row count.
    pub candidate_object_count: usize,
    /// Candidate relation row count.
    pub candidate_relation_count: usize,
    /// Candidate evidence row count.
    pub candidate_evidence_count: usize,
    /// Object rows grouped by candidate kind.
    pub object_kind_counts: Vec<CandidateReadModelKindCount>,
    /// Relation rows grouped by relation kind.
    pub relation_kind_counts: Vec<CandidateReadModelKindCount>,
    /// Evidence rows grouped by evidence kind.
    pub evidence_kind_counts: Vec<CandidateReadModelKindCount>,
    /// Rows whose review status is not review-required.
    pub review_status_violation_count: usize,
    /// Rows whose promotion status is not blocked pending review.
    pub promotion_status_violation_count: usize,
    /// Rows that incorrectly claim ontology truth.
    pub ontology_truth_violation_count: usize,
    /// Object rows that incorrectly allow raw-to-RDF promotion.
    pub raw_to_rdf_promotion_violation_count: usize,
    /// Relation endpoints absent from object rows.
    pub missing_relation_endpoint_count: usize,
    /// Concrete relation endpoint violations.
    pub missing_relation_endpoints: Vec<CandidateReadModelMissingEndpoint>,
    /// Whether all inspection gates passed.
    pub inspection_passed: bool,
}
