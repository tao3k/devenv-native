//! Data contracts for structural-facts reasoning fill plans.

use std::path::{Path, PathBuf};

use serde::Serialize;

pub(super) const REASONING_FILL_PLAN_REPORT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_structural_facts_reasoning_fill_plan_report.v1";

/// Request for compiling a reasoning ledger seed into workflow fill-plan rows.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeOntologyStructuralFactsReasoningFillPlanRequest {
    pub(super) reasoning_ledger_seed_json: PathBuf,
    pub(super) run_id: String,
    pub(super) limit: usize,
}

impl EpistemeOntologyStructuralFactsReasoningFillPlanRequest {
    /// Create a reasoning fill-plan request.
    #[must_use]
    pub fn new(reasoning_ledger_seed_json: impl Into<PathBuf>, run_id: impl Into<String>) -> Self {
        Self {
            reasoning_ledger_seed_json: reasoning_ledger_seed_json.into(),
            run_id: run_id.into(),
            limit: 1024,
        }
    }

    /// Set the maximum number of seed rows to plan.
    #[must_use]
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EpistemeOntologyStructuralFactsReasoningFillPlanItem {
    pub fill_item_id: String,
    pub workflow_key: &'static str,
    pub activity_kind: &'static str,
    pub qianji_activity_contract: &'static str,
    pub seed_id: String,
    pub seed_kind: String,
    pub packet_id: String,
    pub reasoning_task_kind: String,
    pub evidence_target_intent: String,
    pub evidence_anchor_kind: String,
    pub evidence_structure_hint: String,
    pub document_id: String,
    pub document_anchor_id: String,
    pub file_id: String,
    pub domain_id: String,
    pub source_contract_id: String,
    pub relative_path: String,
    pub category: String,
    pub language: String,
    pub extraction_route: String,
    pub source_content_hash: String,
    pub evidence_id: String,
    pub target_ledger_field_group: &'static str,
    pub output_contract: &'static str,
    pub review_decision_required: bool,
    pub promotion_decision_required: bool,
    #[serde(flatten)]
    pub execution: EpistemeOntologyStructuralFactsReasoningFillPlanExecutionFlags,
    #[serde(flatten)]
    pub safety: EpistemeOntologyStructuralFactsReasoningFillPlanSafetyFlags,
    pub status: &'static str,
}

/// Report emitted after reasoning fill-plan generation.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralFactsReasoningFillPlanReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Safe ASCII run id.
    pub run_id: String,
    /// Source reasoning ledger-seed JSON artifact.
    pub reasoning_ledger_seed_json: PathBuf,
    /// Concrete fill-plan run directory.
    pub run_dir: PathBuf,
    /// Generated fill-plan TSV path.
    pub reasoning_fill_plan_tsv: PathBuf,
    /// Generated fill-plan JSON path.
    pub reasoning_fill_plan_json: PathBuf,
    /// Generated fill-plan Org path.
    pub reasoning_fill_plan_org: PathBuf,
    /// Generated report JSON path.
    pub reasoning_fill_plan_report_json: PathBuf,
    /// Number of seed rows consumed.
    pub seed_row_count: usize,
    /// Number of object proposal fill items emitted.
    pub object_fill_item_count: usize,
    /// Number of relation proposal fill items emitted.
    pub relation_fill_item_count: usize,
    /// Number of service-catalog review fill items emitted.
    pub service_catalog_fill_item_count: usize,
    /// Number of object-instance review fill items emitted.
    pub object_instance_fill_item_count: usize,
    /// Total fill-plan items emitted.
    pub fill_item_count: usize,
    /// Number of seed rows skipped by the limit.
    pub skipped_by_limit_count: usize,
    /// Execution safety flags.
    #[serde(flatten)]
    pub execution: EpistemeOntologyStructuralFactsReasoningFillPlanExecutionFlags,
    /// Non-promotion safety flags.
    #[serde(flatten)]
    pub safety: EpistemeOntologyStructuralFactsReasoningFillPlanSafetyFlags,
}

/// Execution flags preserved in fill-plan reports.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralFactsReasoningFillPlanExecutionFlags {
    /// Whether this fill plan read private source text.
    pub source_text_read: bool,
    /// Whether this fill plan called a live LLM.
    pub llm_executed: bool,
    /// Whether this fill plan executed the workflow runtime.
    pub workflow_executed: bool,
}

/// Safety flags preserved in fill-plan reports.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralFactsReasoningFillPlanSafetyFlags {
    /// Whether this fill plan authorizes source mutation.
    pub source_mutation_allowed: bool,
    /// Whether this fill plan authorizes RDF mutation.
    pub rdf_mutation_allowed: bool,
    /// Whether these rows are ontology truth.
    pub ontology_truth: bool,
}

pub(super) struct ReasoningFillPlanOutputPaths {
    pub run_dir: PathBuf,
    pub fill_plan_tsv: PathBuf,
    pub fill_plan_json: PathBuf,
    pub fill_plan_org: PathBuf,
    pub report_json: PathBuf,
}

impl ReasoningFillPlanOutputPaths {
    pub fn new(run_root: &Path, run_key: &str) -> Self {
        let run_dir = run_root.join(run_key);
        Self {
            fill_plan_tsv: run_dir.join("reasoning_fill_plan.tsv"),
            fill_plan_json: run_dir.join("reasoning_fill_plan.json"),
            fill_plan_org: run_dir.join("reasoning_fill_plan.org"),
            report_json: run_dir.join("reasoning_fill_plan_report.json"),
            run_dir,
        }
    }
}
