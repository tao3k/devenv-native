use super::{
    EpistemeCommand, EpistemeRunLegacyOfficeConversionArgs, EpistemeSourceContractCommand,
};

#[test]
fn episteme_source_contract_run_legacy_office_conversion_args_capture_commands() {
    let args = EpistemeRunLegacyOfficeConversionArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("source-contract-registry".to_string()),
        corpus_root: Some("corpus-root".into()),
        run_root: Some("episteme-repo/runs/extraction".into()),
        run_id: "legacy_office_seed".to_string(),
        category: Some("policy".to_string()),
        limit: 4,
        selection_run_id: Some("selection_seed".to_string()),
        selection_root: Some("episteme-repo/runs/evidence-selection".into()),
        converter_command: Some("tools/legacy-office-converter".into()),
        dry_run: true,
    };

    assert_eq!(
        args.episteme_root,
        std::path::PathBuf::from("episteme-repo")
    );
    assert_eq!(args.run_id, "legacy_office_seed");
    assert_eq!(args.category.as_deref(), Some("policy"));
    assert_eq!(args.limit, 4);
    assert_eq!(args.selection_run_id.as_deref(), Some("selection_seed"));
    assert_eq!(
        args.converter_command,
        Some(std::path::PathBuf::from("tools/legacy-office-converter"))
    );
    assert!(args.dry_run);
}

#[test]
fn episteme_source_contract_command_debug_names_legacy_office_variant() {
    let command = EpistemeCommand::SourceContract {
        command: EpistemeSourceContractCommand::RunLegacyOfficeConversion(
            EpistemeRunLegacyOfficeConversionArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                corpus_root: None,
                run_root: None,
                run_id: "legacy_office_seed".to_string(),
                category: None,
                limit: 12,
                selection_run_id: None,
                selection_root: None,
                converter_command: None,
                dry_run: false,
            },
        ),
    };
    let rendered = format!("{command:?}");
    assert!(rendered.contains("SourceContract"));
    assert!(rendered.contains("RunLegacyOfficeConversion"));
}
