use super::{
    EpistemeCommand, EpistemeSourceContractCommand,
    EpistemeWriteStructuralFactsReasoningLedgerSeedArgs,
};

#[test]
fn episteme_source_contract_write_reasoning_ledger_seed_args_capture_input_and_limit() {
    let args = EpistemeWriteStructuralFactsReasoningLedgerSeedArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("source-contract-registry".to_string()),
        reasoning_packet_root: Some("episteme-repo/runs/ontology-generation".into()),
        run_root: Some("episteme-repo/runs/ontology-generation".into()),
        reasoning_packet_run_id: "reasoning_packet".to_string(),
        run_id: "reasoning_ledger_seed".to_string(),
        limit: 32,
    };

    assert_eq!(
        args.reasoning_packet_root,
        Some(std::path::PathBuf::from(
            "episteme-repo/runs/ontology-generation"
        ))
    );
    assert_eq!(args.reasoning_packet_run_id, "reasoning_packet");
    assert_eq!(args.run_id, "reasoning_ledger_seed");
    assert_eq!(args.limit, 32);
}

#[test]
fn episteme_source_contract_command_debug_names_reasoning_ledger_seed_variant() {
    let command = EpistemeCommand::SourceContract {
        command: EpistemeSourceContractCommand::WriteStructuralFactsReasoningLedgerSeed(
            EpistemeWriteStructuralFactsReasoningLedgerSeedArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                reasoning_packet_root: None,
                run_root: None,
                reasoning_packet_run_id: "reasoning_packet".to_string(),
                run_id: "reasoning_ledger_seed".to_string(),
                limit: 512,
            },
        ),
    };

    let rendered = format!("{command:?}");
    assert!(rendered.contains("SourceContract"));
    assert!(rendered.contains("WriteStructuralFactsReasoningLedgerSeed"));
}
