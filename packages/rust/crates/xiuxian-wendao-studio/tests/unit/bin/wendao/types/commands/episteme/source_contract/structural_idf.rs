use super::{
    EpistemeCommand, EpistemeSourceContractCommand, EpistemeStructuralIdfValidationModeArg,
    EpistemeWriteStructuralIdfArgs,
};

#[test]
fn episteme_source_contract_write_structural_idf_args_capture_roots_and_validation() {
    let args = EpistemeWriteStructuralIdfArgs {
        episteme_root: "episteme-repo".into(),
        episteme_registry_id: Some("source-contract-registry".to_string()),
        corpus_root: Some("corpus-root".into()),
        run_root: Some("episteme-repo/runs/structure".into()),
        validation_mode: EpistemeStructuralIdfValidationModeArg::FullHash,
        run_id: "structural_seed".to_string(),
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
        Some(std::path::PathBuf::from("episteme-repo/runs/structure"))
    );
    assert_eq!(
        args.validation_mode,
        EpistemeStructuralIdfValidationModeArg::FullHash
    );
    assert_eq!(args.run_id, "structural_seed");
}

#[test]
fn episteme_source_contract_command_debug_names_structural_idf_variant() {
    let command = EpistemeCommand::SourceContract {
        command: EpistemeSourceContractCommand::WriteStructuralIdf(
            EpistemeWriteStructuralIdfArgs {
                episteme_root: ".".into(),
                episteme_registry_id: None,
                corpus_root: None,
                run_root: None,
                validation_mode: EpistemeStructuralIdfValidationModeArg::MetadataOnly,
                run_id: "structural_seed".to_string(),
            },
        ),
    };

    let rendered = format!("{command:?}");
    assert!(rendered.contains("SourceContract"));
    assert!(rendered.contains("WriteStructuralIdf"));
}
