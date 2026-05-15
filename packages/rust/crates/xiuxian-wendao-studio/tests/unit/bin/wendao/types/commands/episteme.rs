use super::{
    EpistemeCommand, EpistemeEvidenceCommand, EpistemeEvidenceReadValidationModeArg,
    EpistemeEvidenceSelectionValidationModeArg, EpistemePlanExtractionRunArgs,
    EpistemeReadEvidenceArgs, EpistemeSourceContractCommand, EpistemeStructureCommand,
    EpistemeStructureTocValidationModeArg, EpistemeWriteEvidenceSelectionPlanArgs,
    EpistemeWriteStructureTocArgs,
};

#[test]
fn episteme_evidence_read_args_capture_file_id() {
    let args = EpistemeReadEvidenceArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("medical".to_string()),
        corpus_root: Some("corpus-root".into()),
        file_id: "episteme.file.a".to_string(),
        max_preview_bytes: 2048,
        validation_mode: EpistemeEvidenceReadValidationModeArg::FullHash,
    };

    assert_eq!(
        args.episteme_root,
        std::path::PathBuf::from("episteme-repo")
    );
    assert_eq!(args.episteme_registry_id.as_deref(), Some("medical"));
    assert_eq!(
        args.corpus_root,
        Some(std::path::PathBuf::from("corpus-root"))
    );
    assert_eq!(args.file_id, "episteme.file.a");
    assert_eq!(args.max_preview_bytes, 2048);
    assert_eq!(
        args.validation_mode,
        EpistemeEvidenceReadValidationModeArg::FullHash
    );
}

#[test]
fn episteme_evidence_command_debug_names_read_variant() {
    let command = EpistemeCommand::Evidence {
        command: EpistemeEvidenceCommand::Read(EpistemeReadEvidenceArgs {
            episteme_root: ".".into(),
            episteme_registry_id: None,
            corpus_root: None,
            file_id: "episteme.file.a".to_string(),
            max_preview_bytes: 8192,
            validation_mode: EpistemeEvidenceReadValidationModeArg::MetadataOnly,
        }),
    };
    let rendered = format!("{command:?}");
    assert!(rendered.contains("Evidence"));
    assert!(rendered.contains("Read"));
}

#[test]
fn episteme_evidence_selection_plan_args_capture_file_ids() {
    let args = EpistemeWriteEvidenceSelectionPlanArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("medical".to_string()),
        corpus_root: Some("corpus-root".into()),
        run_root: Some("episteme-repo/runs/evidence-selection".into()),
        run_id: "selection_seed".to_string(),
        file_ids: vec!["episteme.file.a".to_string(), "episteme.file.b".to_string()],
        selection_reason: "agent selected source files".to_string(),
        validation_mode: EpistemeEvidenceSelectionValidationModeArg::FullHash,
    };

    assert_eq!(
        args.episteme_root,
        std::path::PathBuf::from("episteme-repo")
    );
    assert_eq!(args.episteme_registry_id.as_deref(), Some("medical"));
    assert_eq!(
        args.corpus_root,
        Some(std::path::PathBuf::from("corpus-root"))
    );
    assert_eq!(
        args.run_root,
        Some(std::path::PathBuf::from(
            "episteme-repo/runs/evidence-selection"
        ))
    );
    assert_eq!(args.run_id, "selection_seed");
    assert_eq!(
        args.file_ids,
        vec!["episteme.file.a".to_string(), "episteme.file.b".to_string()]
    );
    assert_eq!(args.selection_reason, "agent selected source files");
    assert_eq!(
        args.validation_mode,
        EpistemeEvidenceSelectionValidationModeArg::FullHash
    );
}

#[test]
fn episteme_evidence_command_debug_names_selection_variant() {
    let command = EpistemeCommand::Evidence {
        command: EpistemeEvidenceCommand::WriteSelectionPlan(
            EpistemeWriteEvidenceSelectionPlanArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                corpus_root: None,
                run_root: None,
                run_id: "selection_seed".to_string(),
                file_ids: vec!["episteme.file.a".to_string()],
                selection_reason: "manual_or_agent_selected".to_string(),
                validation_mode: EpistemeEvidenceSelectionValidationModeArg::MetadataOnly,
            },
        ),
    };
    let rendered = format!("{command:?}");
    assert!(rendered.contains("Evidence"));
    assert!(rendered.contains("WriteSelectionPlan"));
}

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

#[test]
fn episteme_structure_write_toc_args_capture_roots() {
    let args = EpistemeWriteStructureTocArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("medical".to_string()),
        corpus_root: Some("corpus-root".into()),
        run_root: Some("episteme-repo/runs/structure".into()),
        validation_mode: EpistemeStructureTocValidationModeArg::MetadataOnly,
        run_id: "toc_seed".to_string(),
    };

    assert_eq!(
        args.episteme_root,
        std::path::PathBuf::from("episteme-repo")
    );
    assert_eq!(args.episteme_registry_id.as_deref(), Some("medical"));
    assert_eq!(
        args.corpus_root,
        Some(std::path::PathBuf::from("corpus-root"))
    );
    assert_eq!(
        args.run_root,
        Some(std::path::PathBuf::from("episteme-repo/runs/structure"))
    );
    assert_eq!(
        args.validation_mode,
        EpistemeStructureTocValidationModeArg::MetadataOnly
    );
    assert_eq!(args.run_id, "toc_seed");
}

#[test]
fn episteme_structure_command_debug_names_write_toc_variant() {
    let command = EpistemeCommand::Structure {
        command: EpistemeStructureCommand::WriteToc(EpistemeWriteStructureTocArgs {
            episteme_root: ".".into(),
            episteme_registry_id: None,
            corpus_root: None,
            run_root: None,
            validation_mode: EpistemeStructureTocValidationModeArg::FullHash,
            run_id: "toc_seed".to_string(),
        }),
    };
    let rendered = format!("{command:?}");
    assert!(rendered.contains("Structure"));
    assert!(rendered.contains("WriteToc"));
}
