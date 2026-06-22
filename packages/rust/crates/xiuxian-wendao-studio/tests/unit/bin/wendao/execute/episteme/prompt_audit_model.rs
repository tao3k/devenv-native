use super::{
    DEFAULT_EPISTEME_OPENAI_COMPATIBLE_PROMPT_AUDIT_MODEL, openai_compatible_prompt_audit_model,
};
use crate::bin_support::wendao::types::EpistemeWriteStructuralFactsReasoningQianjiSchedulePlanArgs;

fn qianji_schedule_args() -> EpistemeWriteStructuralFactsReasoningQianjiSchedulePlanArgs {
    EpistemeWriteStructuralFactsReasoningQianjiSchedulePlanArgs {
        episteme_root: ".".into(),
        episteme_registry_id: None,
        fill_plan_root: None,
        run_root: None,
        evidence_extraction_run_root: None,
        fill_plan_run_id: "reasoning_fill_plan".to_string(),
        run_id: "qianji_schedule_plan".to_string(),
        qianji_run_id: None,
        limit: 1024,
        target_ledger_field_group: None,
        evidence_target_intent: None,
        reasoning_context_shard_mode: "disabled".to_string(),
        reasoning_context_shard_row_limit: 2,
        evidence_extraction_run_ids: Vec::new(),
        openai_compatible_model: None,
        openai_compatible_max_tokens: 1024,
    }
}

#[test]
fn prompt_audit_model_defaults_to_deepseek_v4_pro_with_context_evidence() {
    let mut args = qianji_schedule_args();
    args.evidence_extraction_run_ids
        .push("docling_document_cache".to_string());

    assert_eq!(
        openai_compatible_prompt_audit_model(&args).as_deref(),
        Some(DEFAULT_EPISTEME_OPENAI_COMPATIBLE_PROMPT_AUDIT_MODEL)
    );
}

#[test]
fn prompt_audit_model_preserves_explicit_comparator() {
    let mut args = qianji_schedule_args();
    args.evidence_extraction_run_ids
        .push("docling_document_cache".to_string());
    args.openai_compatible_model = Some("comparator/model".to_string());

    assert_eq!(
        openai_compatible_prompt_audit_model(&args).as_deref(),
        Some("comparator/model")
    );
}

#[test]
fn prompt_audit_model_stays_disabled_without_context_evidence() {
    let args = qianji_schedule_args();

    assert_eq!(openai_compatible_prompt_audit_model(&args), None);
}
