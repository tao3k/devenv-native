//! Source-patch semantic preview contracts and row DTOs.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub(super) const SEMANTIC_PREVIEW_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_source_patch_semantic_preview.v1";
pub(super) const SOURCE_PATCH_APPLY_PREVIEW_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_source_patch_apply_preview.v1";
pub(super) const SOURCE_PATCH_APPLY_PLAN_TSV: &str = "source_patch_apply_plan.tsv";
pub(super) const SOURCE_PATCH_APPLY_PREVIEW_JSON: &str = "source_patch_apply_preview.json";
pub(super) const SEMANTIC_OBJECTS_TSV: &str = "semantic_objects.tsv";
pub(super) const SEMANTIC_OBJECTS_JSON: &str = "semantic_objects.json";
pub(super) const SEMANTIC_RELATIONS_TSV: &str = "semantic_relations.tsv";
pub(super) const SEMANTIC_RELATIONS_JSON: &str = "semantic_relations.json";
pub(super) const SEMANTIC_EVIDENCE_TSV: &str = "semantic_evidence.tsv";
pub(super) const SEMANTIC_EVIDENCE_JSON: &str = "semantic_evidence.json";
pub(super) const SEMANTIC_PROJECTION_STATE_JSON: &str = "semantic_projection_state.json";
pub(super) const SEMANTIC_PREVIEW_ORG: &str = "semantic_read_model_preview.org";
pub(super) const SEMANTIC_PREVIEW_JSON: &str = "semantic_read_model_preview.json";
pub(super) const APPLY_ACTION_PROPOSE_TARGETED_SOURCE_PATCH: &str = "propose_targeted_source_patch";
pub(super) const APPROVED_PROMOTION_DECISION: &str = "approved";
pub(super) const OBJECT_INSTANCE_KIND: &str = "object_instance";
pub(super) const INSTANCE_RELATION_KIND: &str = "instance_relation";
pub(super) const ACCEPTED_EVIDENCE_STATUS: &str = "accepted";
pub(super) const ACTIVE_STATUS: &str = "active";
pub(super) const FRESH_STALENESS: &str = "fresh";

/// Request for compiling source-patch preview artifacts into read-model rows.
#[derive(Debug, Clone)]
pub struct EpistemeOntologySourcePatchSemanticPreviewRequest {
    run_dir: PathBuf,
}

impl EpistemeOntologySourcePatchSemanticPreviewRequest {
    /// Create a semantic read-model preview request from a source-patch run.
    #[must_use]
    pub fn new(run_dir: impl Into<PathBuf>) -> Self {
        Self {
            run_dir: run_dir.into(),
        }
    }

    /// Source-patch run directory containing apply-plan and preview artifacts.
    #[must_use]
    pub fn run_dir(&self) -> &Path {
        self.run_dir.as_path()
    }
}

/// Semantic object row compiled from an approved object-instance row.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EpistemeOntologySemanticObjectRow {
    /// Stable semantic object id.
    pub id: String,
    /// Ontology object kind.
    pub kind: String,
    /// Human-facing title or label.
    pub title: String,
    /// Source ontology domain id.
    pub domain: String,
    /// Source evidence id from the review ledger.
    pub evidence_id: String,
    /// Evidence status understood by downstream ontology proof code.
    pub evidence_status: &'static str,
    /// Source target RDF file.
    pub target_rdf_file: String,
    /// Review decision from the ledger.
    pub review_decision: String,
    /// Promotion decision from the ledger.
    pub promotion_decision: String,
    /// Reviewer id from the ledger.
    pub reviewer_id: String,
    /// Number of compiled semantic relations touching this object.
    pub relation_count: usize,
    /// Read-model row status.
    pub status: &'static str,
    /// Projection freshness marker.
    pub read_model_projection_staleness: &'static str,
}

/// Semantic relation row compiled from an approved instance-relation row.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EpistemeOntologySemanticRelationRow {
    /// Stable semantic relation id.
    pub id: String,
    /// Ontology relation kind or predicate.
    pub kind: String,
    /// Source semantic object id.
    pub source: String,
    /// Target semantic object id.
    pub target: String,
    /// Source ontology domain id.
    pub domain: String,
    /// Source evidence id from the review ledger.
    pub evidence_id: String,
    /// Evidence status understood by downstream ontology proof code.
    pub evidence_status: &'static str,
    /// Source target RDF file.
    pub target_rdf_file: String,
    /// Review decision from the ledger.
    pub review_decision: String,
    /// Promotion decision from the ledger.
    pub promotion_decision: String,
    /// Reviewer id from the ledger.
    pub reviewer_id: String,
    /// Read-model row status.
    pub status: &'static str,
    /// Projection freshness marker.
    pub read_model_projection_staleness: &'static str,
}

/// Semantic evidence row preserving row-level provenance.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EpistemeOntologySemanticEvidenceRow {
    /// Stable semantic evidence row id.
    pub id: String,
    /// Original evidence id from the review ledger.
    pub evidence_id: String,
    /// Source apply-plan record id.
    pub record_id: String,
    /// Source apply-plan record kind.
    pub record_kind: String,
    /// Ontology target used by downstream proof code.
    pub ontology_target: String,
    /// Alias for ontology target.
    pub target: String,
    /// Evidence status understood by downstream ontology proof code.
    pub status: &'static str,
    /// Source ontology domain id.
    pub domain: String,
    /// Source target RDF file.
    pub target_rdf_file: String,
    /// Reviewer id from the ledger.
    pub reviewer_id: String,
}

/// Semantic projection state row for the compiled preview.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologySemanticProjectionStateRow {
    /// Projection id.
    pub projection: String,
    /// Projection status.
    pub status: &'static str,
    /// Projection freshness marker.
    pub staleness: &'static str,
    /// Compiled semantic object count.
    pub source_object_count: usize,
    /// Compiled semantic relation count.
    pub source_relation_count: usize,
    /// Compiled evidence row count.
    pub source_evidence_count: usize,
}

/// Report emitted after semantic read-model preview generation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologySourcePatchSemanticPreviewReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Source-patch run directory.
    pub run_dir: PathBuf,
    /// Source apply-plan TSV path.
    pub source_patch_apply_plan_tsv: PathBuf,
    /// Source apply-preview JSON path.
    pub source_patch_apply_preview_json: PathBuf,
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
    /// Generated semantic projection state JSON path.
    pub semantic_projection_state_json: PathBuf,
    /// Generated preview Org receipt path.
    pub semantic_read_model_preview_org: PathBuf,
    /// Generated preview JSON receipt path.
    pub semantic_read_model_preview_json: PathBuf,
    /// Number of apply-plan rows consumed.
    pub apply_plan_row_count: usize,
    /// Number of semantic object rows written.
    pub semantic_object_count: usize,
    /// Number of semantic relation rows written.
    pub semantic_relation_count: usize,
    /// Number of semantic evidence rows written.
    pub semantic_evidence_count: usize,
    /// Number of projection-state rows written.
    pub semantic_projection_state_count: usize,
    /// Whether deterministic projection quality checks passed.
    pub projection_quality_passed: bool,
    /// Deterministic projection quality issues.
    pub quality_issues: Vec<String>,
    /// Whether this preview authorizes source mutation.
    pub source_mutation_allowed: bool,
    /// Whether this preview marks rows as ontology truth.
    pub ontology_truth: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SourcePatchApplyPreviewReceipt {
    pub(super) schema_version: String,
    pub(super) apply_plan_tsv_sha256: String,
    pub(super) apply_plan_row_count: usize,
    pub(super) preview_targets: Vec<SourcePatchApplyPreviewTarget>,
    pub(super) source_mutation_allowed: bool,
    pub(super) ontology_truth: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SourcePatchApplyPreviewTarget {
    pub(super) target_rdf_file: String,
    pub(super) proposed_rdf_admission_passed: bool,
    pub(super) preview_row_count: usize,
}

#[derive(Debug, Clone)]
pub(super) struct SourcePatchApplyPlanRow {
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
