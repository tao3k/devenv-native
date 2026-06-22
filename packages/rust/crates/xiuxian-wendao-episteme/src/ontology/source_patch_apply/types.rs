//! Source-patch apply contracts and internal DTOs.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub(crate) const SOURCE_PATCH_APPLY_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_source_patch_apply.v1";
pub(crate) const SOURCE_PATCH_REVIEW_PACKET_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_ontology_source_patch_review_packet.v1";
pub(crate) const SOURCE_PATCH_APPLY_PLAN_TSV: &str = "source_patch_apply_plan.tsv";
pub(crate) const SOURCE_PATCH_REVIEW_PACKET_JSON: &str = "source_patch_review_packet.json";
pub(crate) const SOURCE_PATCH_APPLY_ORG: &str = "source_patch_apply.org";
pub(crate) const SOURCE_PATCH_APPLY_JSON: &str = "source_patch_apply.json";
pub(crate) const APPLY_ACTION_PROPOSE_TARGETED_SOURCE_PATCH: &str = "propose_targeted_source_patch";
pub(crate) const OBJECT_INSTANCE_KIND: &str = "object_instance";
pub(crate) const INSTANCE_RELATION_KIND: &str = "instance_relation";
pub(crate) const WDSP_NS: &str = "https://wendao.ai/ontology/source-patch#";
pub(crate) const BEGIN_BLOCK: &str = "BEGIN WENDAO SOURCE PATCH";
pub(crate) const END_BLOCK: &str = "END WENDAO SOURCE PATCH";

/// Request for applying a reviewed source patch to ontology source files.
#[derive(Debug, Clone)]
pub struct EpistemeOntologySourcePatchApplyRequest {
    episteme_root: PathBuf,
    run_dir: PathBuf,
    pub(crate) expected_apply_plan_tsv_sha256: Option<String>,
    pub(crate) allow_source_mutation: bool,
}

impl EpistemeOntologySourcePatchApplyRequest {
    /// Create a source-patch apply request.
    #[must_use]
    pub fn new(episteme_root: impl Into<PathBuf>, run_dir: impl Into<PathBuf>) -> Self {
        Self {
            episteme_root: episteme_root.into(),
            run_dir: run_dir.into(),
            expected_apply_plan_tsv_sha256: None,
            allow_source_mutation: false,
        }
    }

    /// Require the operator-observed apply-plan TSV hash.
    #[must_use]
    pub fn with_expected_apply_plan_tsv_sha256(mut self, expected: impl Into<String>) -> Self {
        self.expected_apply_plan_tsv_sha256 = Some(expected.into());
        self
    }

    /// Explicitly enable source mutation.
    #[must_use]
    pub fn with_allow_source_mutation(mut self, allow: bool) -> Self {
        self.allow_source_mutation = allow;
        self
    }

    /// Episteme repository root containing ontology source files.
    #[must_use]
    pub fn episteme_root(&self) -> &Path {
        self.episteme_root.as_path()
    }

    /// Source-patch run directory containing reviewed artifacts.
    #[must_use]
    pub fn run_dir(&self) -> &Path {
        self.run_dir.as_path()
    }
}

/// Hash metadata for one source-patch target after application.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologySourcePatchAppliedTarget {
    /// Target RDF path relative to the Episteme `ontology/` directory.
    pub target_rdf_file: String,
    /// SHA-256 digest recorded in the review packet before mutation.
    pub before_rdf_sha256: String,
    /// SHA-256 digest after writing the source-patch block.
    pub after_rdf_sha256: String,
    /// Number of apply-plan rows written to this target.
    pub applied_row_count: usize,
}

/// Report emitted after source-patch application.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologySourcePatchApplyReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Episteme repository root used to resolve target RDF files.
    pub episteme_root: PathBuf,
    /// Source-patch run directory.
    pub run_dir: PathBuf,
    /// Source review-packet JSON path.
    pub source_patch_review_packet_json: PathBuf,
    /// Source apply-plan TSV path.
    pub source_patch_apply_plan_tsv: PathBuf,
    /// Generated apply receipt Org path.
    pub source_patch_apply_org: PathBuf,
    /// Generated apply receipt JSON path.
    pub source_patch_apply_json: PathBuf,
    /// Operator-provided expected apply-plan TSV hash.
    pub expected_apply_plan_tsv_sha256: String,
    /// Actual apply-plan TSV hash.
    pub apply_plan_tsv_sha256: String,
    /// Number of apply-plan rows applied.
    pub apply_plan_row_count: usize,
    /// Number of target RDF files mutated.
    pub target_rdf_file_count: usize,
    /// Per-target mutation receipts.
    pub applied_targets: Vec<EpistemeOntologySourcePatchAppliedTarget>,
    /// Whether this request authorized source mutation.
    pub source_mutation_allowed: bool,
    /// Whether the applied proposal block itself is ontology truth.
    pub ontology_truth: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SourcePatchReviewPacketReceipt {
    pub(crate) schema_version: String,
    pub(crate) apply_plan_tsv_sha256: String,
    pub(crate) apply_plan_row_count: usize,
    pub(crate) object_apply_plan_count: usize,
    pub(crate) relation_apply_plan_count: usize,
    pub(crate) target_rdf_files: Vec<SourcePatchReviewPacketTarget>,
    pub(crate) source_mutation_allowed: bool,
    pub(crate) ontology_truth: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SourcePatchReviewPacketTarget {
    pub(crate) target_rdf_file: String,
    pub(crate) target_rdf_sha256: String,
}

#[derive(Debug, Clone)]
pub(crate) struct SourcePatchApplyPlanRow {
    pub(crate) record_id: String,
    pub(crate) record_kind: String,
    pub(crate) domain_id: String,
    pub(crate) target_rdf_file: String,
    pub(crate) label: String,
    pub(crate) object_type: String,
    pub(crate) source_object_id: String,
    pub(crate) predicate: String,
    pub(crate) target_object_id: String,
    pub(crate) evidence_id: String,
    pub(crate) review_decision: String,
    pub(crate) promotion_decision: String,
    pub(crate) reviewer_id: String,
    pub(crate) apply_action: String,
    pub(crate) source_mutation_allowed: bool,
    pub(crate) ontology_truth: bool,
}

pub(crate) struct TargetWritePlan {
    pub(crate) target_rdf_file: String,
    pub(crate) path: PathBuf,
    pub(crate) before_hash: String,
    pub(crate) proposed_content: String,
    pub(crate) proposal_block: String,
    pub(crate) row_count: usize,
}

pub(crate) struct ReviewedSourcePatchArtifacts {
    pub(crate) source_patch_review_packet_json: PathBuf,
    pub(crate) source_patch_apply_plan_tsv: PathBuf,
    pub(crate) expected_apply_plan_tsv_sha256: String,
    pub(crate) apply_plan_tsv_sha256: String,
    pub(crate) apply_plan_row_count: usize,
    pub(crate) write_plans: Vec<TargetWritePlan>,
}
