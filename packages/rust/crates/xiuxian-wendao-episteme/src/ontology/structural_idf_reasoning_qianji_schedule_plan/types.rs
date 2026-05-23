use std::path::{Path, PathBuf};

use serde::Serialize;

pub(super) const QIANJI_SCHEDULE_PLAN_REPORT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_structural_idf_reasoning_qianji_schedule_plan_report.v1";

/// Request for compiling a reasoning fill plan into Qianji schedule inputs.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanRequest {
    pub(super) reasoning_fill_plan_json: PathBuf,
    pub(super) run_id: String,
    pub(super) qianji_run_id: Option<String>,
    pub(super) limit: usize,
    pub(super) openai_compatible_prompt_audit:
        Option<EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanPromptAudit>,
}

impl EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanRequest {
    /// Create a Qianji schedule-plan request.
    #[must_use]
    pub fn new(reasoning_fill_plan_json: impl Into<PathBuf>, run_id: impl Into<String>) -> Self {
        Self {
            reasoning_fill_plan_json: reasoning_fill_plan_json.into(),
            run_id: run_id.into(),
            qianji_run_id: None,
            limit: 1024,
            openai_compatible_prompt_audit: None,
        }
    }

    /// Set the Qianji run id carried by generated activity tasks.
    #[must_use]
    pub fn with_qianji_run_id(mut self, qianji_run_id: impl Into<String>) -> Self {
        self.qianji_run_id = Some(qianji_run_id.into());
        self
    }

    /// Set the maximum number of fill-plan rows to schedule.
    #[must_use]
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Emit OpenAI-compatible prompt audit metadata for Qianji execution.
    #[must_use]
    pub fn with_openai_compatible_prompt_audit(
        mut self,
        model: impl Into<String>,
        max_tokens: u32,
    ) -> Self {
        self.openai_compatible_prompt_audit = Some(
            EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanPromptAudit {
                model: model.into(),
                max_tokens,
            },
        );
        self
    }
}

/// OpenAI-compatible prompt audit controls for generated Qianji tasks.
#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanPromptAudit {
    pub model: String,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanItem {
    pub schedule_item_id: String,
    pub schedule_contract: &'static str,
    pub admission_kind: &'static str,
    pub qianji_run_id: String,
    pub fill_item_id: String,
    pub workflow_key: String,
    pub activity_kind: String,
    pub seed_id: String,
    pub seed_kind: String,
    pub packet_id: String,
    pub document_id: String,
    pub document_anchor_id: String,
    pub file_id: String,
    pub evidence_id: String,
    pub field_group: String,
    pub activity_task: QianjiActivityTaskShape,
    #[serde(flatten)]
    pub execution: EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanExecutionFlags,
    #[serde(flatten)]
    pub safety: EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanSafetyFlags,
    pub status: &'static str,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub(super) struct QianjiActivityTaskShape {
    pub activity_id: String,
    pub activity_type: String,
    pub task_queue: String,
    pub input_ref: QianjiArtifactRefShape,
    pub idempotency_key: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub(super) struct QianjiArtifactRefShape {
    pub artifact_id: String,
    pub artifact_kind: String,
    pub uri: String,
    pub content_digest: String,
    pub metadata: serde_json::Value,
}

/// Report emitted after Qianji schedule-plan generation.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanReport {
    /// Report schema identifier.
    pub schema_version: &'static str,
    /// Safe ASCII run id.
    pub run_id: String,
    /// Qianji run id carried by generated task payloads.
    pub qianji_run_id: String,
    /// Source reasoning fill-plan JSON artifact.
    pub reasoning_fill_plan_json: PathBuf,
    /// Concrete schedule-plan run directory.
    pub run_dir: PathBuf,
    /// Generated schedule-plan TSV path.
    pub qianji_schedule_plan_tsv: PathBuf,
    /// Generated schedule-plan JSON path.
    pub qianji_schedule_plan_json: PathBuf,
    /// Generated schedule-plan Org path.
    pub qianji_schedule_plan_org: PathBuf,
    /// Generated report JSON path.
    pub qianji_schedule_plan_report_json: PathBuf,
    /// Number of fill-plan rows consumed.
    pub fill_item_count: usize,
    /// Number of object proposal schedule items emitted.
    pub object_schedule_item_count: usize,
    /// Number of relation proposal schedule items emitted.
    pub relation_schedule_item_count: usize,
    /// Total schedule items emitted.
    pub schedule_item_count: usize,
    /// Number of fill-plan rows skipped by the limit.
    pub skipped_by_limit_count: usize,
    /// Execution safety flags.
    #[serde(flatten)]
    pub execution: EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanExecutionFlags,
    /// Non-promotion safety flags.
    #[serde(flatten)]
    pub safety: EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanSafetyFlags,
}

/// Execution flags preserved in Qianji schedule-plan reports.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanExecutionFlags {
    /// Source/model execution flags.
    #[serde(flatten)]
    pub input: EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanInputExecutionFlags,
    /// Runtime execution flags.
    #[serde(flatten)]
    pub runtime: EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanRuntimeExecutionFlags,
}

impl EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanExecutionFlags {
    pub(super) const fn inactive() -> Self {
        Self {
            input: EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanInputExecutionFlags {
                source_text_read: false,
                llm_executed: false,
            },
            runtime:
                EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanRuntimeExecutionFlags {
                    workflow_executed: false,
                    qianji_ledger_mutated: false,
                    hot_state_enqueued: false,
                },
        }
    }
}

/// Source/model execution flags preserved in Qianji schedule-plan reports.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanInputExecutionFlags {
    /// Whether this schedule plan read private source text.
    pub source_text_read: bool,
    /// Whether this schedule plan called a live LLM.
    pub llm_executed: bool,
}

/// Runtime execution flags preserved in Qianji schedule-plan reports.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanRuntimeExecutionFlags {
    /// Whether this schedule plan executed the workflow runtime.
    pub workflow_executed: bool,
    /// Whether this schedule plan wrote Qianji control ledger events.
    pub qianji_ledger_mutated: bool,
    /// Whether this schedule plan enqueued hot-state work.
    pub hot_state_enqueued: bool,
}

/// Safety flags preserved in Qianji schedule-plan reports.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralIdfReasoningQianjiSchedulePlanSafetyFlags {
    /// Whether this schedule plan authorizes source mutation.
    pub source_mutation_allowed: bool,
    /// Whether this schedule plan authorizes RDF mutation.
    pub rdf_mutation_allowed: bool,
    /// Whether these rows are ontology truth.
    pub ontology_truth: bool,
}

pub(super) struct QianjiSchedulePlanOutputPaths {
    pub run_dir: PathBuf,
    pub prompt_artifact_dir: PathBuf,
    pub context_artifact_dir: PathBuf,
    pub schedule_plan_tsv: PathBuf,
    pub schedule_plan_json: PathBuf,
    pub schedule_plan_org: PathBuf,
    pub report_json: PathBuf,
}

impl QianjiSchedulePlanOutputPaths {
    pub fn new(run_root: &Path, run_id: &str) -> Self {
        let run_dir = run_root.join(run_id);
        Self {
            prompt_artifact_dir: run_dir.join("prompt_artifacts"),
            context_artifact_dir: run_dir.join("context_artifacts"),
            schedule_plan_tsv: run_dir.join("qianji_schedule_plan.tsv"),
            schedule_plan_json: run_dir.join("qianji_schedule_plan.json"),
            schedule_plan_org: run_dir.join("qianji_schedule_plan.org"),
            report_json: run_dir.join("qianji_schedule_plan_report.json"),
            run_dir,
        }
    }
}
