use super::{
    EpistemeBootstrapPipelineArgs, EpistemeCommand, EpistemePlanExtractionRunArgs,
    EpistemeSourceContractCommand,
};

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
fn episteme_source_contract_bootstrap_pipeline_args_capture_artifact_roots() {
    let args = EpistemeBootstrapPipelineArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("source-contract-registry".to_string()),
        corpus_root: Some("corpus-root".into()),
        structure_run_root: Some("episteme-repo/runs/structure".into()),
        ontology_generation_run_root: Some("episteme-repo/runs/ontology-generation".into()),
        validation_mode: Default::default(),
        run_id: "bootstrap_seed".to_string(),
        category: Some("policy".to_string()),
        route: Some("document_text_evidence".to_string()),
        reasoning_packet_limit: 16,
        reasoning_ledger_seed_limit: 32,
        reasoning_fill_plan_limit: 64,
        #[cfg(feature = "episteme-foyer-artifact-cache")]
        artifact_cache_mode: Default::default(),
        #[cfg(feature = "episteme-foyer-artifact-cache")]
        artifact_cache_source_digest: None,
        #[cfg(feature = "episteme-foyer-artifact-cache")]
        artifact_cache_profile_digest: None,
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
        args.structure_run_root,
        Some(std::path::PathBuf::from("episteme-repo/runs/structure"))
    );
    assert_eq!(
        args.ontology_generation_run_root,
        Some(std::path::PathBuf::from(
            "episteme-repo/runs/ontology-generation"
        ))
    );
    assert_eq!(args.run_id, "bootstrap_seed");
    assert_eq!(args.category.as_deref(), Some("policy"));
    assert_eq!(args.route.as_deref(), Some("document_text_evidence"));
    assert_eq!(args.reasoning_packet_limit, 16);
    assert_eq!(args.reasoning_ledger_seed_limit, 32);
    assert_eq!(args.reasoning_fill_plan_limit, 64);
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

#[test]
fn episteme_source_contract_command_debug_names_bootstrap_pipeline_variant() {
    let command = EpistemeCommand::SourceContract {
        command: EpistemeSourceContractCommand::BootstrapPipeline(EpistemeBootstrapPipelineArgs {
            episteme_root: ".".into(),
            episteme_registry_id: None,
            corpus_root: None,
            structure_run_root: None,
            ontology_generation_run_root: None,
            validation_mode: Default::default(),
            run_id: "bootstrap_seed".to_string(),
            category: None,
            route: None,
            reasoning_packet_limit: 256,
            reasoning_ledger_seed_limit: 512,
            reasoning_fill_plan_limit: 1024,
            #[cfg(feature = "episteme-foyer-artifact-cache")]
            artifact_cache_mode: Default::default(),
            #[cfg(feature = "episteme-foyer-artifact-cache")]
            artifact_cache_source_digest: None,
            #[cfg(feature = "episteme-foyer-artifact-cache")]
            artifact_cache_profile_digest: None,
        }),
    };
    let rendered = format!("{command:?}");
    assert!(rendered.contains("SourceContract"));
    assert!(rendered.contains("BootstrapPipeline"));
}
