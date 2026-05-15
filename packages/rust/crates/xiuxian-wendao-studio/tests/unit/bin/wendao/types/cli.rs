use super::Cli;
use crate::bin_support::wendao::types::{
    Command, EpistemeCommand, EpistemeEvidenceCommand, EpistemeEvidenceReadValidationModeArg,
    EpistemeEvidenceSelectionValidationModeArg, EpistemeSourceContractCommand,
    EpistemeStructureCommand, EpistemeStructureTocValidationModeArg,
};
use clap::Parser;

#[test]
fn parses_embedded_client_lint_command() {
    let cli = Cli::parse_from(["wendao", "lint", "markdown", "README.md"]);
    assert!(matches!(cli.command, Command::Client(_)));
}

#[test]
fn parses_get_toc_command() {
    let cli = Cli::parse_from(["wendao", "get", "toc", "docs/guides"]);
    assert!(matches!(cli.command, Command::Client(_)));
}

#[test]
fn parses_get_page_index_command() {
    let cli = Cli::parse_from(["wendao", "get", "page-index"]);
    assert!(matches!(cli.command, Command::Client(_)));
}

#[test]
fn parses_audit_load_episteme_command() {
    let cli = Cli::parse_from(["wendao", "audit", "--load", "wendao-episteme", "docs"]);

    let Command::Audit(args) = cli.command else {
        panic!("expected audit command");
    };

    assert_eq!(args.target, "docs");
    assert_eq!(args.load.as_deref(), Some("wendao-episteme"));
}

#[test]
fn parses_audit_template_command() {
    let cli = Cli::parse_from(["wendao", "audit", "--template", "johnny-decimal"]);

    let Command::Audit(args) = cli.command else {
        panic!("expected audit command");
    };

    assert_eq!(args.target, ".");
    assert_eq!(args.template.as_deref(), Some("johnny-decimal"));
}

#[test]
fn parses_episteme_source_contract_plan_extraction_run_selection_run_id_command() {
    let cli = Cli::parse_from([
        "wendao",
        "episteme",
        "source-contract",
        "plan-extraction-run",
        "--episteme-root",
        "source-contract",
        "--episteme-registry-id",
        "configured-source",
        "--corpus-root",
        "corpus-root",
        "--run-id",
        "source_contract_seed",
        "--route",
        "document_text_evidence",
        "--limit",
        "3",
        "--selection-run-id",
        "selection_seed",
        "--selection-root",
        "source-contract/runs/evidence-selection",
    ]);

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::SourceContract { command } = command else {
        panic!("expected episteme source-contract command");
    };
    let EpistemeSourceContractCommand::PlanExtractionRun(args) = command;
    assert_eq!(
        args.episteme_root,
        std::path::PathBuf::from("source-contract")
    );
    assert_eq!(
        args.episteme_registry_id.as_deref(),
        Some("configured-source")
    );
    assert_eq!(
        args.corpus_root,
        Some(std::path::PathBuf::from("corpus-root"))
    );
    assert_eq!(args.run_id, "source_contract_seed");
    assert_eq!(args.route.as_deref(), Some("document_text_evidence"));
    assert_eq!(args.limit, 3);
    assert_eq!(args.selection_run_id.as_deref(), Some("selection_seed"));
    assert_eq!(
        args.selection_root,
        Some(std::path::PathBuf::from(
            "source-contract/runs/evidence-selection"
        ))
    );
}

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
