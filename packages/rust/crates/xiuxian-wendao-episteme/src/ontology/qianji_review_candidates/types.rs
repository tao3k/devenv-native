//! Qianji review import contracts, payload DTOs, and row types.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::ontology::candidate_review::EpistemeOntologyCandidateReviewReport;

pub(super) const IMPORT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_qianji_review_candidate_import.v1";
pub(super) const QIANJI_RESPONSE_SCHEMA: &str = "qianji.openai_compatible_llm_response.v1";
pub(super) const EPISTEME_REVIEW_SCHEMA: &str = "xiuxian.wendao.episteme.reasoning_fill_review.v1";
pub(super) const OBJECTS_TSV: &str = "candidate_objects.tsv";
pub(super) const RELATIONS_TSV: &str = "candidate_relations.tsv";
pub(super) const EVIDENCE_TSV: &str = "candidate_evidence.tsv";
pub(super) const IMPORT_REPORT_JSON: &str = "qianji_review_candidate_import_report.json";
pub(super) const OBJECT_MODEL_OBJECT_TYPE_PATCH_KIND: &str = "object_model_object_type_candidate";
pub(super) const OBJECT_MODEL_LINK_TYPE_PATCH_KIND: &str = "object_model_link_type_candidate";

/// Request for importing Qianji Episteme review artifacts as candidate rows.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeOntologyQianjiReviewCandidateImportRequest {
    pub(super) run_dir: PathBuf,
    pub(super) review_artifacts: Vec<PathBuf>,
}

impl EpistemeOntologyQianjiReviewCandidateImportRequest {
    /// Create an import request for the ontology-generation run directory.
    #[must_use]
    pub fn new(run_dir: impl Into<PathBuf>) -> Self {
        Self {
            run_dir: run_dir.into(),
            review_artifacts: Vec::new(),
        }
    }

    /// Add a Qianji OpenAI-compatible review artifact to import.
    #[must_use]
    pub fn with_review_artifact(mut self, path: impl Into<PathBuf>) -> Self {
        self.review_artifacts.push(path.into());
        self
    }

    /// Ontology-generation run directory receiving candidate rows.
    #[must_use]
    pub fn run_dir(&self) -> &Path {
        self.run_dir.as_path()
    }
}

/// Report emitted after importing Qianji review artifacts.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyQianjiReviewCandidateImportReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Run directory receiving candidate rows and review outputs.
    pub run_dir: PathBuf,
    /// Source Qianji review artifacts imported.
    pub qianji_review_artifacts: Vec<PathBuf>,
    /// Generated candidate object TSV path.
    pub candidate_objects_tsv: PathBuf,
    /// Generated candidate relation TSV path.
    pub candidate_relations_tsv: PathBuf,
    /// Generated candidate evidence TSV path.
    pub candidate_evidence_tsv: PathBuf,
    /// Generated import report path.
    pub import_report_json: PathBuf,
    /// Number of imported object candidates.
    pub candidate_object_count: usize,
    /// Number of imported relation candidates.
    pub candidate_relation_count: usize,
    /// Number of imported evidence rows.
    pub candidate_evidence_count: usize,
    /// Number of canonical review artifacts that produced no candidate rows.
    pub zero_candidate_review_count: usize,
    /// Total number of model-declared review blockers.
    pub review_blocker_count: usize,
    /// Review-gate report over the imported candidates.
    pub candidate_review: EpistemeOntologyCandidateReviewReport,
    /// Whether imported rows are ontology truth.
    pub ontology_truth: bool,
    /// Whether imported rows allow raw-to-RDF promotion.
    pub raw_to_rdf_promotion_allowed: bool,
}

#[derive(Debug, Deserialize)]
pub(super) struct QianjiReviewArtifact {
    pub(super) schema: String,
    #[serde(default)]
    pub(super) episteme_review: Option<EpistemeReview>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EpistemeReview {
    pub(super) schema: String,
    pub(super) status: String,
    pub(super) fill_item_id: String,
    pub(super) target_ledger_field_group: String,
    #[serde(default)]
    pub(super) blockers: Vec<String>,
    pub(super) candidate_patch_count: usize,
    pub(super) candidate_patches: Vec<EpistemeCandidatePatch>,
    pub(super) rdf_mutation: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EpistemeCandidatePatch {
    pub(super) patch_kind: String,
    #[serde(default)]
    pub(super) fill_item_id: String,
    #[serde(default)]
    pub(super) target_ledger_field_group: String,
    #[serde(default)]
    pub(super) provisional_object_key: String,
    #[serde(default)]
    pub(super) provisional_relation_key: String,
    #[serde(default)]
    pub(super) ontology_class_key: String,
    #[serde(default)]
    pub(super) relation_property_key: String,
    #[serde(default)]
    pub(super) label: String,
    #[serde(default)]
    pub(super) source_object_label: String,
    #[serde(default)]
    pub(super) target_object_label: String,
    #[serde(default)]
    pub(super) object_type: Option<EpistemeObjectModelObjectTypePatch>,
    #[serde(default)]
    pub(super) link_type: Option<EpistemeObjectModelLinkTypePatch>,
    #[serde(default)]
    pub(super) endpoint_object_types: Vec<EpistemeObjectModelEndpointObjectTypePatch>,
    #[serde(default)]
    pub(super) source_evidence: Vec<EpistemePatchEvidence>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EpistemeObjectModelObjectTypePatch {
    pub(super) api_name: String,
    pub(super) display_name: String,
    #[serde(default)]
    pub(super) rdf_class: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EpistemeObjectModelLinkTypePatch {
    pub(super) api_name: String,
    pub(super) display_name: String,
    pub(super) rdf_property: String,
    pub(super) from_object_type: String,
    pub(super) to_object_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EpistemeObjectModelEndpointObjectTypePatch {
    pub(super) api_name: String,
    pub(super) display_name: String,
    #[serde(default)]
    pub(super) rdf_class: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EpistemePatchEvidence {
    pub(super) file_id: String,
    #[serde(default)]
    pub(super) relative_path: String,
    pub(super) quote: String,
}

#[derive(Debug)]
pub(super) struct CandidateObjectRow {
    pub(super) candidate_id: String,
    pub(super) label: String,
    pub(super) suggested_term_key: String,
    pub(super) source_file_id: String,
    pub(super) source_path: String,
    pub(super) evidence_sha256: String,
    pub(super) text_char_count: usize,
}

#[derive(Debug)]
pub(super) struct CandidateRelationRow {
    pub(super) candidate_id: String,
    pub(super) relation_kind: String,
    pub(super) source_candidate_id: String,
    pub(super) target_candidate_id: String,
    pub(super) source_file_id: String,
    pub(super) evidence_sha256: String,
}

#[derive(Debug)]
pub(super) struct CandidateEvidenceRow {
    pub(super) evidence_id: String,
    pub(super) source_file_id: String,
    pub(super) source_path: String,
    pub(super) evidence_sha256: String,
    pub(super) text_char_count: usize,
}

#[derive(Default)]
pub(super) struct QianjiReviewCandidateImportBuild {
    pub(super) objects: Vec<CandidateObjectRow>,
    pub(super) relations: Vec<CandidateRelationRow>,
    pub(super) evidence: Vec<CandidateEvidenceRow>,
    pub(super) zero_candidate_review_count: usize,
    pub(super) review_blocker_count: usize,
}
