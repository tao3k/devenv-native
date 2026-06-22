use super::{
    EpistemeCommand, EpistemeSourceContractCommand,
    EpistemeWriteStructuralFactsReasoningQianjiSchedulePlanArgs,
};

#[test]
fn episteme_source_contract_write_qianji_schedule_plan_args_capture_input_and_limit() {
    let args = EpistemeWriteStructuralFactsReasoningQianjiSchedulePlanArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("source-contract-registry".to_string()),
        fill_plan_root: Some("episteme-repo/runs/ontology-generation".into()),
        run_root: Some("episteme-repo/runs/ontology-generation".into()),
        evidence_extraction_run_root: Some("episteme-repo/runs/extraction".into()),
        fill_plan_run_id: "reasoning_fill_plan".to_string(),
        run_id: "qianji_schedule_plan".to_string(),
        qianji_run_id: Some("episteme.ontology.reasoning.qianji".to_string()),
        limit: 64,
        target_ledger_field_group: Some("service_catalog_review".to_string()),
        evidence_target_intent: Some("service_catalog_extraction".to_string()),
        reasoning_context_shard_mode: "service-catalog-table-rows".to_string(),
        reasoning_context_shard_row_limit: 2,
        evidence_extraction_run_ids: vec!["ltc_docling_real_probe".to_string()],
        openai_compatible_model: Some("deepseek/deepseek-v4-pro".to_string()),
        openai_compatible_max_tokens: 768,
    };

    assert_eq!(
        args.fill_plan_root,
        Some(std::path::PathBuf::from(
            "episteme-repo/runs/ontology-generation"
        ))
    );
    assert_eq!(args.fill_plan_run_id, "reasoning_fill_plan");
    assert_eq!(args.run_id, "qianji_schedule_plan");
    assert_eq!(
        args.evidence_extraction_run_root,
        Some(std::path::PathBuf::from("episteme-repo/runs/extraction"))
    );
    assert_eq!(
        args.evidence_extraction_run_ids,
        vec!["ltc_docling_real_probe".to_string()]
    );
    assert_eq!(
        args.qianji_run_id,
        Some("episteme.ontology.reasoning.qianji".to_string())
    );
    assert_eq!(args.limit, 64);
    assert_eq!(
        args.target_ledger_field_group.as_deref(),
        Some("service_catalog_review")
    );
    assert_eq!(
        args.evidence_target_intent.as_deref(),
        Some("service_catalog_extraction")
    );
    assert_eq!(
        args.reasoning_context_shard_mode,
        "service-catalog-table-rows"
    );
    assert_eq!(args.reasoning_context_shard_row_limit, 2);
    assert_eq!(
        args.openai_compatible_model,
        Some("deepseek/deepseek-v4-pro".to_string())
    );
    assert_eq!(args.openai_compatible_max_tokens, 768);
}

#[test]
fn episteme_source_contract_command_debug_names_qianji_schedule_plan_variant() {
    let command = EpistemeCommand::SourceContract {
        command: EpistemeSourceContractCommand::WriteStructuralFactsReasoningQianjiSchedulePlan(
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
            },
        ),
    };

    let rendered = format!("{command:?}");
    assert!(rendered.contains("SourceContract"));
    assert!(rendered.contains("WriteStructuralFactsReasoningQianjiSchedulePlan"));
}
