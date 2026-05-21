use super::{
    EpistemeCommand, EpistemeStructureCommand, EpistemeStructureTocValidationModeArg,
    EpistemeWriteStructureTocArgs,
};

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
