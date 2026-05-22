use super::{EpistemeCommand, EpistemePlanExtractionRunArgs, EpistemeSourceContractCommand};

#[test]
fn episteme_source_contract_plan_extraction_run_selection_run_id_args_capture_filters() {
    let args = EpistemePlanExtractionRunArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("source-contract-registry".to_string()),
        corpus_root: Some("corpus-root".into()),
        run_root: Some("episteme-repo/runs/extraction".into()),
        run_id: "source_contract_seed".to_string(),
        route: Some("document_text_evidence".to_string()),
        category: Some("policy".to_string()),
        limit: 12,
        selection_run_id: Some("selection_seed".to_string()),
        selection_root: Some("episteme-repo/runs/evidence-selection".into()),
    };
    assert_eq!(
        args.episteme_root,
        std::path::PathBuf::from("episteme-repo")
    );
    assert_eq!(
        args.episteme_registry_id.as_deref(),
        Some("source-contract-registry")
    );
    assert_eq!(
        args.corpus_root,
        Some(std::path::PathBuf::from("corpus-root"))
    );
    assert_eq!(
        args.run_root,
        Some(std::path::PathBuf::from("episteme-repo/runs/extraction"))
    );
    assert_eq!(args.run_id, "source_contract_seed");
    assert_eq!(args.route.as_deref(), Some("document_text_evidence"));
    assert_eq!(args.category.as_deref(), Some("policy"));
    assert_eq!(args.limit, 12);
    assert_eq!(args.selection_run_id.as_deref(), Some("selection_seed"));
    assert_eq!(
        args.selection_root,
        Some(std::path::PathBuf::from(
            "episteme-repo/runs/evidence-selection"
        ))
    );
}

#[test]
fn episteme_source_contract_command_debug_names_plan_variant() {
    let command = EpistemeCommand::SourceContract {
        command: EpistemeSourceContractCommand::PlanExtractionRun(EpistemePlanExtractionRunArgs {
            episteme_root: ".".into(),
            episteme_registry_id: None,
            corpus_root: None,
            run_root: None,
            run_id: "source_contract_seed".to_string(),
            route: None,
            category: None,
            limit: 12,
            selection_run_id: None,
            selection_root: None,
        }),
    };
    let rendered = format!("{command:?}");
    assert!(rendered.contains("SourceContract"));
    assert!(rendered.contains("PlanExtractionRun"));
}
