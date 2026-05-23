use super::{
    EpistemeCommand, EpistemeSourceContractCommand,
    EpistemeWriteStructuralIdfReasoningQianjiSchedulePlanArgs,
};

#[test]
fn episteme_source_contract_write_qianji_schedule_plan_args_capture_input_and_limit() {
    let args = EpistemeWriteStructuralIdfReasoningQianjiSchedulePlanArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("source-contract-registry".to_string()),
        fill_plan_root: Some("episteme-repo/runs/ontology-generation".into()),
        run_root: Some("episteme-repo/runs/ontology-generation".into()),
        fill_plan_run_id: "reasoning_fill_plan".to_string(),
        run_id: "qianji_schedule_plan".to_string(),
        qianji_run_id: Some("episteme.ontology.reasoning.qianji".to_string()),
        limit: 64,
        openai_compatible_model: Some("openrouter/deepseek/deepseek-chat-v3.1".to_string()),
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
        args.qianji_run_id,
        Some("episteme.ontology.reasoning.qianji".to_string())
    );
    assert_eq!(args.limit, 64);
    assert_eq!(
        args.openai_compatible_model,
        Some("openrouter/deepseek/deepseek-chat-v3.1".to_string())
    );
    assert_eq!(args.openai_compatible_max_tokens, 768);
}

#[test]
fn episteme_source_contract_command_debug_names_qianji_schedule_plan_variant() {
    let command = EpistemeCommand::SourceContract {
        command: EpistemeSourceContractCommand::WriteStructuralIdfReasoningQianjiSchedulePlan(
            EpistemeWriteStructuralIdfReasoningQianjiSchedulePlanArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                fill_plan_root: None,
                run_root: None,
                fill_plan_run_id: "reasoning_fill_plan".to_string(),
                run_id: "qianji_schedule_plan".to_string(),
                qianji_run_id: None,
                limit: 1024,
                openai_compatible_model: None,
                openai_compatible_max_tokens: 1024,
            },
        ),
    };

    let rendered = format!("{command:?}");
    assert!(rendered.contains("SourceContract"));
    assert!(rendered.contains("WriteStructuralIdfReasoningQianjiSchedulePlan"));
}
