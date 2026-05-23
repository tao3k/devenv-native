use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub(super) const APPLY_ACTION_PROPOSE_TARGETED_SOURCE_PATCH: &str = "propose_targeted_source_patch";
pub(super) const APPROVED_PROMOTION_DECISION: &str = "approved";
pub(super) const OBJECT_INSTANCE_KIND: &str = "object_instance";
pub(super) const INSTANCE_RELATION_KIND: &str = "instance_relation";
pub(super) const ACCEPTED_EVIDENCE_STATUS: &str = "accepted";
pub(super) const ACTIVE_STATUS: &str = "active";
pub(super) const FRESH_STALENESS: &str = "fresh";

pub(super) const SOURCE_PATCH_APPLY_JSON: &str = "source_patch_apply.json";
pub(super) const RDF_SOURCE_SEMANTIC_OBJECTS_TSV: &str = "rdf_source_semantic_objects.tsv";
pub(super) const RDF_SOURCE_SEMANTIC_OBJECTS_JSON: &str = "rdf_source_semantic_objects.json";
pub(super) const RDF_SOURCE_SEMANTIC_RELATIONS_TSV: &str = "rdf_source_semantic_relations.tsv";
pub(super) const RDF_SOURCE_SEMANTIC_RELATIONS_JSON: &str = "rdf_source_semantic_relations.json";
pub(super) const RDF_SOURCE_SEMANTIC_EVIDENCE_TSV: &str = "rdf_source_semantic_evidence.tsv";
pub(super) const RDF_SOURCE_SEMANTIC_EVIDENCE_JSON: &str = "rdf_source_semantic_evidence.json";
pub(super) const RDF_SOURCE_PROJECTION_STATE_JSON: &str = "rdf_source_projection_state.json";
pub(super) const RDF_SOURCE_READ_MODEL_ORG: &str = "rdf_source_read_model.org";
pub(super) const RDF_SOURCE_READ_MODEL_JSON: &str = "rdf_source_read_model.json";

#[derive(Debug, Clone)]
pub(super) struct SourcePatchRdfRow {
    pub(super) record_id: String,
    pub(super) record_kind: String,
    pub(super) domain_id: String,
    pub(super) target_rdf_file: String,
    pub(super) label: String,
    pub(super) object_type: String,
    pub(super) source_object_id: String,
    pub(super) predicate: String,
    pub(super) target_object_id: String,
    pub(super) evidence_id: String,
    pub(super) review_decision: String,
    pub(super) promotion_decision: String,
    pub(super) reviewer_id: String,
    pub(super) apply_action: String,
    pub(super) source_mutation_allowed: bool,
    pub(super) ontology_truth: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SourcePatchApplyReceipt {
    pub(super) schema_version: String,
    pub(super) source_patch_apply_json: PathBuf,
    pub(super) apply_plan_row_count: usize,
    pub(super) target_rdf_file_count: usize,
    pub(super) applied_targets: Vec<SourcePatchAppliedTargetReceipt>,
    pub(super) source_mutation_allowed: bool,
    pub(super) ontology_truth: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SourcePatchAppliedTargetReceipt {
    pub(super) target_rdf_file: String,
    pub(super) after_rdf_sha256: String,
    pub(super) applied_row_count: usize,
}

/// Request for compiling applied source-patch RDF into semantic read-model rows.
#[derive(Debug, Clone)]
pub struct EpistemeOntologySourcePatchRdfReadModelRequest {
    episteme_root: PathBuf,
    run_dir: PathBuf,
}

impl EpistemeOntologySourcePatchRdfReadModelRequest {
    /// Create a source-patch RDF read-model request.
    #[must_use]
    pub fn new(episteme_root: impl Into<PathBuf>, run_dir: impl Into<PathBuf>) -> Self {
        Self {
            episteme_root: episteme_root.into(),
            run_dir: run_dir.into(),
        }
    }

    /// Episteme repository root containing ontology source files.
    #[must_use]
    pub fn episteme_root(&self) -> &std::path::Path {
        self.episteme_root.as_path()
    }

    /// Source-patch run directory containing the apply receipt.
    #[must_use]
    pub fn run_dir(&self) -> &std::path::Path {
        self.run_dir.as_path()
    }
}

/// Report emitted after RDF-source semantic read-model generation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologySourcePatchRdfReadModelReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Episteme repository root used to resolve target RDF files.
    pub episteme_root: PathBuf,
    /// Source-patch run directory.
    pub run_dir: PathBuf,
    /// Source apply receipt JSON path.
    pub source_patch_apply_json: PathBuf,
    /// Generated semantic objects TSV path.
    pub semantic_objects_tsv: PathBuf,
    /// Generated semantic objects JSON path.
    pub semantic_objects_json: PathBuf,
    /// Generated semantic relations TSV path.
    pub semantic_relations_tsv: PathBuf,
    /// Generated semantic relations JSON path.
    pub semantic_relations_json: PathBuf,
    /// Generated semantic evidence TSV path.
    pub semantic_evidence_tsv: PathBuf,
    /// Generated semantic evidence JSON path.
    pub semantic_evidence_json: PathBuf,
    /// Generated projection-state JSON path.
    pub semantic_projection_state_json: PathBuf,
    /// Generated Org receipt path.
    pub rdf_source_read_model_org: PathBuf,
    /// Generated JSON receipt path.
    pub rdf_source_read_model_json: PathBuf,
    /// Number of source-patch RDF rows parsed from target RDF source.
    pub rdf_source_row_count: usize,
    /// Number of semantic object rows written.
    pub semantic_object_count: usize,
    /// Number of semantic relation rows written.
    pub semantic_relation_count: usize,
    /// Number of semantic evidence rows written.
    pub semantic_evidence_count: usize,
    /// Number of projection-state rows written.
    pub semantic_projection_state_count: usize,
    /// Number of target RDF files read.
    pub target_rdf_file_count: usize,
    /// Whether deterministic projection quality checks passed.
    pub projection_quality_passed: bool,
    /// Deterministic projection quality issues.
    pub quality_issues: Vec<String>,
    /// Whether the source apply receipt authorized mutation.
    pub source_mutation_allowed: bool,
    /// Whether the RDF source rows are ontology truth.
    pub ontology_truth: bool,
}
