use super::{
    EpistemeCommand, EpistemeSourceContractCommand, EpistemeWriteStructuralFactsReasoningPacketArgs,
};

#[test]
fn episteme_source_contract_write_structural_facts_reasoning_packet_args_capture_input_and_filters()
{
    let args = EpistemeWriteStructuralFactsReasoningPacketArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("source-contract-registry".to_string()),
        structure_run_root: Some("episteme-repo/runs/structure".into()),
        run_root: Some("episteme-repo/runs/ontology-generation".into()),
        structural_facts_run_id: "structural_seed".to_string(),
        run_id: "reasoning_seed".to_string(),
        category: Some("policy".to_string()),
        route: Some("document_text_evidence".to_string()),
        limit: 16,
    };

    assert_eq!(
        args.structure_run_root,
        Some(std::path::PathBuf::from("episteme-repo/runs/structure"))
    );
    assert_eq!(args.structural_facts_run_id, "structural_seed");
    assert_eq!(args.run_id, "reasoning_seed");
    assert_eq!(args.category.as_deref(), Some("policy"));
    assert_eq!(args.route.as_deref(), Some("document_text_evidence"));
    assert_eq!(args.limit, 16);
}

#[test]
fn episteme_source_contract_command_debug_names_reasoning_packet_variant() {
    let command = EpistemeCommand::SourceContract {
        command: EpistemeSourceContractCommand::WriteStructuralFactsReasoningPacket(
            EpistemeWriteStructuralFactsReasoningPacketArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                structure_run_root: None,
                run_root: None,
                structural_facts_run_id: "structural_seed".to_string(),
                run_id: "reasoning_seed".to_string(),
                category: None,
                route: None,
                limit: 256,
            },
        ),
    };

    let rendered = format!("{command:?}");
    assert!(rendered.contains("SourceContract"));
    assert!(rendered.contains("WriteStructuralFactsReasoningPacket"));
}
