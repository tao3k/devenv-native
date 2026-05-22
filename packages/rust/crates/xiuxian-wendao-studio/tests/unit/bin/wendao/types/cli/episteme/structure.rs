use super::{
    Cli, Command, EpistemeCommand, EpistemeStructureCommand, EpistemeStructureTocValidationModeArg,
    Parser,
};

#[test]
fn parses_episteme_structure_write_toc_command() {
    let cli = Cli::parse_from([
        "wendao",
        "episteme",
        "structure",
        "write-toc",
        "--episteme-registry-id",
        "medical",
        "--corpus-root",
        "corpus-root",
        "--validation-mode",
        "full-hash",
        "--run-id",
        "toc_seed",
    ]);

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::Structure { command } = command else {
        panic!("expected episteme structure command");
    };
    let EpistemeStructureCommand::WriteToc(args) = command;
    assert_eq!(args.episteme_root, std::path::PathBuf::from("."));
    assert_eq!(args.episteme_registry_id.as_deref(), Some("medical"));
    assert_eq!(
        args.corpus_root,
        Some(std::path::PathBuf::from("corpus-root"))
    );
    assert_eq!(
        args.validation_mode,
        EpistemeStructureTocValidationModeArg::FullHash
    );
    assert_eq!(args.run_id, "toc_seed");
}
