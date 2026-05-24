use super::{
    EpistemeCommand, EpistemeSourceContractCommand,
    EpistemeWriteStructuralFactsReasoningFillPlanArgs,
};

#[test]
fn episteme_source_contract_write_reasoning_fill_plan_args_capture_input_and_limit() {
    let args = EpistemeWriteStructuralFactsReasoningFillPlanArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("source-contract-registry".to_string()),
        ledger_seed_root: Some("episteme-repo/runs/ontology-generation".into()),
        run_root: Some("episteme-repo/runs/ontology-generation".into()),
        ledger_seed_run_id: "reasoning_ledger_seed".to_string(),
        run_id: "reasoning_fill_plan".to_string(),
        limit: 64,
    };

    assert_eq!(
        args.ledger_seed_root,
        Some(std::path::PathBuf::from(
            "episteme-repo/runs/ontology-generation"
        ))
    );
    assert_eq!(args.ledger_seed_run_id, "reasoning_ledger_seed");
    assert_eq!(args.run_id, "reasoning_fill_plan");
    assert_eq!(args.limit, 64);
}

#[test]
fn episteme_source_contract_command_debug_names_reasoning_fill_plan_variant() {
    let command = EpistemeCommand::SourceContract {
        command: EpistemeSourceContractCommand::WriteStructuralFactsReasoningFillPlan(
            EpistemeWriteStructuralFactsReasoningFillPlanArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                ledger_seed_root: None,
                run_root: None,
                ledger_seed_run_id: "reasoning_ledger_seed".to_string(),
                run_id: "reasoning_fill_plan".to_string(),
                limit: 1024,
            },
        ),
    };

    let rendered = format!("{command:?}");
    assert!(rendered.contains("SourceContract"));
    assert!(rendered.contains("WriteStructuralFactsReasoningFillPlan"));
}
