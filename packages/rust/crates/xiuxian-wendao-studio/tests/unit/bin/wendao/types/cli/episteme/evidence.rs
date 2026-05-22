use super::{
    Cli, Command, EpistemeCommand, EpistemeEvidenceCommand, EpistemeEvidenceReadValidationModeArg,
    EpistemeEvidenceSelectionValidationModeArg, Parser,
};

#[test]
fn parses_episteme_evidence_read_command() {
    let cli = Cli::parse_from([
        "wendao",
        "episteme",
        "evidence",
        "read",
        "--episteme-registry-id",
        "medical",
        "--corpus-root",
        "corpus-root",
        "--file-id",
        "episteme.file.a",
        "--max-preview-bytes",
        "1024",
        "--validation-mode",
        "full-hash",
    ]);

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::Evidence { command } = command else {
        panic!("expected episteme evidence command");
    };
    let EpistemeEvidenceCommand::Read(args) = command else {
        panic!("expected evidence read command");
    };
    assert_eq!(args.episteme_root, std::path::PathBuf::from("."));
    assert_eq!(args.episteme_registry_id.as_deref(), Some("medical"));
    assert_eq!(
        args.corpus_root,
        Some(std::path::PathBuf::from("corpus-root"))
    );
    assert_eq!(args.file_id, "episteme.file.a");
    assert_eq!(args.max_preview_bytes, 1024);
    assert_eq!(
        args.validation_mode,
        EpistemeEvidenceReadValidationModeArg::FullHash
    );
}

#[test]
fn parses_episteme_evidence_selection_plan_command() {
    let cli = Cli::parse_from([
        "wendao",
        "episteme",
        "evidence",
        "write-selection-plan",
        "--episteme-registry-id",
        "medical",
        "--corpus-root",
        "corpus-root",
        "--run-id",
        "selection_seed",
        "--file-id",
        "episteme.file.a",
        "--file-id",
        "episteme.file.b",
        "--selection-reason",
        "agent selected source files",
        "--validation-mode",
        "full-hash",
    ]);

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::Evidence { command } = command else {
        panic!("expected episteme evidence command");
    };
    let EpistemeEvidenceCommand::WriteSelectionPlan(args) = command else {
        panic!("expected write-selection-plan command");
    };
    assert_eq!(args.episteme_root, std::path::PathBuf::from("."));
    assert_eq!(args.episteme_registry_id.as_deref(), Some("medical"));
    assert_eq!(
        args.corpus_root,
        Some(std::path::PathBuf::from("corpus-root"))
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
fn parses_episteme_runtime_config_defaulted_selection_command_without_corpus_root() {
    let cli = Cli::parse_from([
        "wendao",
        "episteme",
        "evidence",
        "write-selection-plan",
        "--episteme-root",
        "medical-episteme",
        "--run-id",
        "selection_seed",
        "--file-id",
        "episteme.file.a",
    ]);

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::Evidence { command } = command else {
        panic!("expected episteme evidence command");
    };
    let EpistemeEvidenceCommand::WriteSelectionPlan(args) = command else {
        panic!("expected write-selection-plan command");
    };
    assert_eq!(
        args.episteme_root,
        std::path::PathBuf::from("medical-episteme")
    );
    assert!(args.corpus_root.is_none());
    assert!(args.run_root.is_none());
    assert_eq!(args.run_id, "selection_seed");
    assert_eq!(args.file_ids, vec!["episteme.file.a".to_string()]);
}
