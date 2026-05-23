use super::{Cli, Command, EpistemeCommand, EpistemeSourceContractCommand, Parser};

#[test]
fn parses_episteme_source_contract_plan_extraction_run_selection_run_id_command() {
    let cli = Cli::parse_from([
        "wendao",
        "episteme",
        "source-contract",
        "plan-extraction-run",
        "--episteme-root",
        "source-contract",
        "--episteme-registry-id",
        "configured-source",
        "--corpus-root",
        "corpus-root",
        "--run-id",
        "source_contract_seed",
        "--route",
        "document_text_evidence",
        "--limit",
        "3",
        "--selection-run-id",
        "selection_seed",
        "--selection-root",
        "source-contract/runs/evidence-selection",
    ]);

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::SourceContract { command } = command else {
        panic!("expected episteme source-contract command");
    };
    let EpistemeSourceContractCommand::PlanExtractionRun(args) = command else {
        panic!("expected episteme source-contract plan-extraction-run command");
    };
    assert_eq!(
        args.episteme_root,
        std::path::PathBuf::from("source-contract")
    );
    assert_eq!(
        args.episteme_registry_id.as_deref(),
        Some("configured-source")
    );
    assert_eq!(
        args.corpus_root,
        Some(std::path::PathBuf::from("corpus-root"))
    );
    assert_eq!(args.run_id, "source_contract_seed");
    assert_eq!(args.route.as_deref(), Some("document_text_evidence"));
    assert_eq!(args.limit, 3);
    assert_eq!(args.selection_run_id.as_deref(), Some("selection_seed"));
    assert_eq!(
        args.selection_root,
        Some(std::path::PathBuf::from(
            "source-contract/runs/evidence-selection"
        ))
    );
}

#[test]
fn parses_episteme_source_contract_write_qianji_schedule_plan_prompt_audit_command() {
    let cli = Cli::parse_from([
        "wendao",
        "episteme",
        "source-contract",
        "write-structural-idf-reasoning-qianji-schedule-plan",
        "--episteme-root",
        "source-contract",
        "--fill-plan-run-id",
        "reasoning_fill_plan",
        "--run-id",
        "qianji_schedule_plan",
        "--qianji-run-id",
        "episteme.ontology.reasoning.test",
        "--limit",
        "1",
        "--openai-compatible-model",
        "openrouter/deepseek/deepseek-chat-v3.1",
        "--openai-compatible-max-tokens",
        "768",
    ]);

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::SourceContract { command } = command else {
        panic!("expected episteme source-contract command");
    };
    let EpistemeSourceContractCommand::WriteStructuralIdfReasoningQianjiSchedulePlan(args) =
        command
    else {
        panic!("expected qianji schedule-plan command");
    };

    assert_eq!(
        args.episteme_root,
        std::path::PathBuf::from("source-contract")
    );
    assert_eq!(args.fill_plan_run_id, "reasoning_fill_plan");
    assert_eq!(args.run_id, "qianji_schedule_plan");
    assert_eq!(
        args.qianji_run_id.as_deref(),
        Some("episteme.ontology.reasoning.test")
    );
    assert_eq!(args.limit, 1);
    assert_eq!(
        args.openai_compatible_model.as_deref(),
        Some("openrouter/deepseek/deepseek-chat-v3.1")
    );
    assert_eq!(args.openai_compatible_max_tokens, 768);
}
