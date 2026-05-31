//! Data contracts for Qianji schedule plans derived from reasoning fill plans.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::ontology::reasoning_context_shard::REASONING_CONTEXT_SHARD_MODE_DISABLED;

pub(super) const QIANJI_SCHEDULE_PLAN_REPORT_SCHEMA_VERSION: &str =
    "xiuxian_wendao.episteme_structural_facts_reasoning_qianji_schedule_plan_report.v1";

/// Request for compiling a reasoning fill plan into Qianji schedule inputs.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRequest {
    pub(super) reasoning_fill_plan_json: PathBuf,
    pub(super) run_id: String,
    pub(super) qianji_run_id: Option<String>,
    pub(super) limit: usize,
    pub(super) target_ledger_field_group: Option<String>,
    pub(super) evidence_target_intent: Option<String>,
    pub(super) reasoning_context_shard_mode: String,
    pub(super) reasoning_context_shard_row_limit: usize,
    pub(super) evidence_extraction_run_root: Option<PathBuf>,
    pub(super) evidence_extraction_run_ids: Vec<String>,
    pub(super) openai_compatible_prompt_audit:
        Option<EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanPromptAudit>,
}

impl EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRequest {
    /// Create a Qianji schedule-plan request.
    #[must_use]
    pub fn new(reasoning_fill_plan_json: impl Into<PathBuf>, run_id: impl Into<String>) -> Self {
        Self {
            reasoning_fill_plan_json: reasoning_fill_plan_json.into(),
            run_id: run_id.into(),
            qianji_run_id: None,
            limit: 1024,
            target_ledger_field_group: None,
            evidence_target_intent: None,
            reasoning_context_shard_mode: REASONING_CONTEXT_SHARD_MODE_DISABLED.to_owned(),
            reasoning_context_shard_row_limit: 2,
            evidence_extraction_run_root: None,
            evidence_extraction_run_ids: Vec::new(),
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

    /// Restrict scheduling to fill-plan rows with this target ledger field group.
    #[must_use]
    pub fn with_target_ledger_field_group(
        mut self,
        target_ledger_field_group: impl Into<String>,
    ) -> Self {
        self.target_ledger_field_group = Some(target_ledger_field_group.into());
        self
    }

    /// Restrict scheduling to fill-plan rows with this evidence target intent.
    #[must_use]
    pub fn with_evidence_target_intent(
        mut self,
        evidence_target_intent: impl Into<String>,
    ) -> Self {
        self.evidence_target_intent = Some(evidence_target_intent.into());
        self
    }

    /// Set the deterministic reasoning context sharding mode used for prompt-audit tasks.
    #[must_use]
    pub fn with_reasoning_context_shard_mode(mut self, mode: impl Into<String>) -> Self {
        self.reasoning_context_shard_mode = mode.into();
        self
    }

    /// Set the maximum number of table data rows per reasoning context shard.
    #[must_use]
    pub fn with_reasoning_context_shard_row_limit(mut self, row_limit: usize) -> Self {
        self.reasoning_context_shard_row_limit = row_limit;
        self
    }

    /// Set the extraction-run root used to materialize context evidence.
    #[must_use]
    pub fn with_evidence_extraction_run_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.evidence_extraction_run_root = Some(root.into());
        self
    }

    /// Add an extraction run whose cache outputs should be included in context.
    #[must_use]
    pub fn with_evidence_extraction_run_id(mut self, run_id: impl Into<String>) -> Self {
        self.evidence_extraction_run_ids.push(run_id.into());
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
            EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanPromptAudit {
                model: model.into(),
                max_tokens,
            },
        );
        self
    }
}

/// OpenAI-compatible prompt audit controls for generated Qianji tasks.
#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanPromptAudit {
    pub model: String,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanItem {
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
    pub evidence_target_intent: String,
    pub evidence_anchor_kind: String,
    pub evidence_structure_hint: String,
    pub document_id: String,
    pub document_anchor_id: String,
    pub file_id: String,
    pub evidence_id: String,
    pub field_group: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_context_shard_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_context_shard_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_context_shard_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_context_shard_row_start: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_context_shard_row_end: Option<usize>,
    pub activity_task: QianjiActivityTaskShape,
    #[serde(flatten)]
    pub execution: EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanExecutionFlags,
    #[serde(flatten)]
    pub safety: EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanSafetyFlags,
    pub status: &'static str,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub(super) struct QianjiActivityTaskShape {
    pub activity_id: String,
    pub activity_type: String,
    pub task_queue: String,
    pub input_ref: QianjiArtifactRefShape,
    pub idempotency_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_policy: Option<QianjiActivityRetryPolicyShape>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub(super) struct QianjiActivityRetryPolicyShape {
    pub max_attempts: u32,
    pub initial_interval_ms: u64,
    pub backoff_multiplier_millis: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub non_retryable_error_codes: Vec<String>,
}

impl QianjiActivityRetryPolicyShape {
    pub(super) fn llm_provider_default() -> Self {
        Self {
            max_attempts: 2,
            initial_interval_ms: 1_000,
            backoff_multiplier_millis: 2_000,
            non_retryable_error_codes: Vec::new(),
        }
    }
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
pub struct EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanReport {
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
    /// Number of service-catalog review schedule items emitted.
    pub service_catalog_schedule_item_count: usize,
    /// Number of object-instance review schedule items emitted.
    pub object_instance_schedule_item_count: usize,
    /// Total schedule items emitted.
    pub schedule_item_count: usize,
    /// Number of fill-plan rows skipped by the limit.
    pub skipped_by_limit_count: usize,
    /// Number of fill-plan rows skipped by target filters.
    pub skipped_by_filter_count: usize,
    /// Reasoning context sharding mode applied while generating prompt-audit tasks.
    pub reasoning_context_shard_mode: String,
    /// Maximum table data rows allowed in each reasoning context shard.
    pub reasoning_context_shard_row_limit: usize,
    /// Number of reasoning context shards emitted as schedule items.
    pub reasoning_context_shard_count: usize,
    /// Optional target ledger field group filter applied before limit selection.
    pub target_ledger_field_group: Option<String>,
    /// Optional evidence target intent filter applied before limit selection.
    pub evidence_target_intent: Option<String>,
    /// Extraction runs used to materialize context evidence.
    pub context_evidence_run_ids: Vec<String>,
    /// Number of cache evidence rows materialized into context artifacts.
    pub context_evidence_item_count: usize,
    /// Number of scheduled fill items without materialized context evidence.
    pub context_evidence_missing_item_count: usize,
    /// Execution safety flags.
    #[serde(flatten)]
    pub execution: EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanExecutionFlags,
    /// Non-promotion safety flags.
    #[serde(flatten)]
    pub safety: EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanSafetyFlags,
}

/// Execution flags preserved in Qianji schedule-plan reports.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanExecutionFlags {
    /// Source/model execution flags.
    #[serde(flatten)]
    pub input: EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanInputExecutionFlags,
    /// Runtime execution flags.
    #[serde(flatten)]
    pub runtime: EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRuntimeExecutionFlags,
}

impl EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanExecutionFlags {
    pub(super) const fn inactive() -> Self {
        Self {
            input: EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanInputExecutionFlags {
                source_text_read: false,
                llm_executed: false,
            },
            runtime:
                EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRuntimeExecutionFlags {
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
pub struct EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanInputExecutionFlags {
    /// Whether this schedule plan read private source text.
    pub source_text_read: bool,
    /// Whether this schedule plan called a live LLM.
    pub llm_executed: bool,
}

/// Runtime execution flags preserved in Qianji schedule-plan reports.
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanRuntimeExecutionFlags {
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
pub struct EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanSafetyFlags {
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
    pub fn new(run_root: &Path, run_key: &str) -> Self {
        let run_dir = run_root.join(run_key);
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
