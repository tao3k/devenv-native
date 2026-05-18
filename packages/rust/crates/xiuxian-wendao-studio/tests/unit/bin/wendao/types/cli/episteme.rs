use super::{
    Cli, Command, EpistemeCommand, EpistemeEvidenceCommand, EpistemeEvidenceReadValidationModeArg,
    EpistemeEvidenceSelectionValidationModeArg, EpistemeSourceContractCommand,
    EpistemeStructureCommand, EpistemeStructureTocValidationModeArg, Parser,
};

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
    let EpistemeSourceContractCommand::PlanExtractionRun(args) = command else {
        panic!("expected episteme source-contract plan-extraction-run command");
    };
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
fn parses_episteme_source_contract_run_image_ocr_cache_command() {
    let cli = Cli::parse_from([
        "wendao",
        "episteme",
        "source-contract",
        "run-image-ocr-cache",
        "--episteme-root",
        "source-contract",
        "--episteme-registry-id",
        "configured-source",
        "--corpus-root",
        "corpus-root",
        "--run-id",
        "ltc_image_ocr_seed",
        "--category",
        "wechat_image",
        "--limit",
        "4",
        "--selection-run-id",
        "selection_seed",
        "--selection-root",
        "source-contract/runs/evidence-selection",
        "--analyzer-command",
        "wendao-image-ocr-jsonl",
        "--ocr-results-jsonl",
        "source-contract/runs/extraction/ltc_image_ocr_seed/ocr_results.jsonl",
        "--dry-run",
    ]);

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::SourceContract { command } = command else {
        panic!("expected episteme source-contract command");
    };
    let EpistemeSourceContractCommand::RunImageOcrCache(args) = command else {
        panic!("expected episteme source-contract run-image-ocr-cache command");
    };
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
    assert_eq!(args.run_id, "ltc_image_ocr_seed");
    assert_eq!(args.category.as_deref(), Some("wechat_image"));
    assert_eq!(args.limit, 4);
    assert_eq!(args.selection_run_id.as_deref(), Some("selection_seed"));
    assert_eq!(
        args.selection_root,
        Some(std::path::PathBuf::from(
            "source-contract/runs/evidence-selection"
        ))
    );
    assert_eq!(args.analyzer_command, "wendao-image-ocr-jsonl");
    assert_eq!(
        args.ocr_results_jsonl,
        Some(std::path::PathBuf::from(
            "source-contract/runs/extraction/ltc_image_ocr_seed/ocr_results.jsonl"
        ))
    );
    assert!(args.dry_run);
}

#[test]
fn parses_episteme_source_contract_run_docling_document_cache_command() {
    let cli = Cli::parse_from([
        "wendao",
        "episteme",
        "source-contract",
        "run-docling-document-cache",
        "--episteme-root",
        "source-contract",
        "--episteme-registry-id",
        "configured-source",
        "--corpus-root",
        "corpus-root",
        "--run-id",
        "ltc_docling_document_seed",
        "--category",
        "policy",
        "--limit",
        "4",
        "--selection-run-id",
        "selection_seed",
        "--selection-root",
        "source-contract/runs/evidence-selection",
        "--analyzer-command",
        "wendao-docling-document-jsonl",
        "--docling-profile",
        "full",
        "--document-results-jsonl",
        "source-contract/runs/extraction/ltc_docling_document_seed/document_results.jsonl",
        "--dry-run",
    ]);

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::SourceContract { command } = command else {
        panic!("expected episteme source-contract command");
    };
    let EpistemeSourceContractCommand::RunDoclingDocumentCache(args) = command else {
        panic!("expected episteme source-contract run-docling-document-cache command");
    };
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
    assert_eq!(args.run_id, "ltc_docling_document_seed");
    assert_eq!(args.category.as_deref(), Some("policy"));
    assert_eq!(args.limit, 4);
    assert_eq!(args.selection_run_id.as_deref(), Some("selection_seed"));
    assert_eq!(
        args.selection_root,
        Some(std::path::PathBuf::from(
            "source-contract/runs/evidence-selection"
        ))
    );
    assert_eq!(args.analyzer_command, "wendao-docling-document-jsonl");
    assert_eq!(args.docling_profile, "full");
    assert_eq!(
        args.document_results_jsonl,
        Some(std::path::PathBuf::from(
            "source-contract/runs/extraction/ltc_docling_document_seed/document_results.jsonl"
        ))
    );
    assert!(args.dry_run);
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
