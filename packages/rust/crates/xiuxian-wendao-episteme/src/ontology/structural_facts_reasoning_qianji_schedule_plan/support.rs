//! Qianji schedule-plan constants and build DTOs.

use super::{
    evidence::{ContextEvidenceByFileId, ContextEvidenceRow},
    types::{
        EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanItem,
        EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanPromptAudit,
        QianjiArtifactRefShape, QianjiSchedulePlanOutputPaths,
    },
};
use crate::ontology::reasoning_context_shard::EpistemeReasoningContextShard;

pub(super) const SCHEDULE_CONTRACT: &str =
    "xiuxian.qianji.control.activity_schedule_admission_plan.v1";
pub(super) const ADMISSION_KIND: &str = "qianji_activity_schedule_admission_candidate";
pub(super) const ACTIVITY_TYPE: &str = "episteme.ontology.reasoning_fill";
pub(super) const TASK_QUEUE: &str = "episteme.ontology.reasoning";
pub(super) const INPUT_ARTIFACT_KIND: &str = "episteme.reasoning_fill_item";
pub(super) const STATUS_PENDING: &str = "pending_qianji_admission";
pub(super) const PROMPT_ARTIFACT_KIND: &str = "llm.prompt";
pub(super) const CONTEXT_ARTIFACT_KIND: &str = "episteme.reasoning_fill_context";
pub(super) const QIANJI_LLM_ACTIVITY_REQUEST_AUDIT_SCHEMA: &str =
    "qianji.llm_activity_request_audit.v1";
pub(super) const TARGET_CONTRACT_SCHEMA: &str =
    "xiuxian.wendao.episteme.reasoning_target_contract.v1";
pub(super) const OBJECT_MODEL_SCHEMA_REF: &str =
    "https://wendao.ai/schema/episteme/object-model-v1.schema.json";
pub(super) const OBJECT_MODEL_COMPATIBILITY: &str = "foundry_style_object_model_v1";
pub(super) const OBJECT_MODEL_TARGET_LAYER: &str = "object_model";
pub(super) const RDF_SOURCE_AUTHORITY: &str = "rdf";
pub(super) const OBJECT_MODEL_OBJECT_TYPE_PATCH_KIND: &str = "object_model_object_type_candidate";
pub(super) const OBJECT_MODEL_LINK_TYPE_PATCH_KIND: &str = "object_model_link_type_candidate";

pub(super) struct ScheduleItemSelection {
    pub(super) items: Vec<EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanItem>,
    pub(super) selected_fill_item_count: usize,
    pub(super) skipped_by_limit_count: usize,
    pub(super) skipped_by_filter_count: usize,
}

pub(super) struct ScheduleBuildOptions<'a> {
    pub(super) schedule_run_id: &'a str,
    pub(super) qianji_run_id: &'a str,
    pub(super) limit: usize,
    pub(super) target_ledger_field_group: Option<&'a str>,
    pub(super) evidence_target_intent: Option<&'a str>,
    pub(super) reasoning_context_shard_mode: &'a str,
    pub(super) reasoning_context_shard_row_limit: usize,
    pub(super) paths: &'a QianjiSchedulePlanOutputPaths,
    pub(super) prompt_audit:
        Option<&'a EpistemeOntologyStructuralFactsReasoningQianjiSchedulePlanPromptAudit>,
    pub(super) context_evidence_by_file_id: &'a ContextEvidenceByFileId,
}

pub(super) struct ScheduleItemContext {
    pub(super) reasoning_context_shard: Option<EpistemeReasoningContextShard>,
    pub(super) context_evidence: Vec<ContextEvidenceRow>,
}

pub(super) struct PromptAuditArtifacts {
    pub(super) prompt_ref: QianjiArtifactRefShape,
    pub(super) request_audit: serde_json::Value,
}
