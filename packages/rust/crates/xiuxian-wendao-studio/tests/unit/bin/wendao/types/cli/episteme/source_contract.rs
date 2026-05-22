use super::{Cli, Command, EpistemeCommand, EpistemeSourceContractCommand, Parser};

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
        "--use-existing-results",
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
    assert_eq!(args.analyzer_command, "wendao-image-ocr-jsonl");
    assert!(args.use_existing_results);
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
        "--use-existing-results",
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
    assert_eq!(args.run_id, "ltc_docling_document_seed");
    assert_eq!(args.category.as_deref(), Some("policy"));
    assert_eq!(args.limit, 4);
    assert_eq!(args.analyzer_command, "wendao-docling-document-jsonl");
    assert_eq!(args.docling_profile, "full");
    assert!(args.use_existing_results);
    assert!(args.dry_run);
}

#[test]
fn parses_episteme_source_contract_run_legacy_office_conversion_command() {
    let cli = Cli::parse_from([
        "wendao",
        "episteme",
        "source-contract",
        "run-legacy-office-conversion",
        "--episteme-root",
        "source-contract",
        "--episteme-registry-id",
        "configured-source",
        "--corpus-root",
        "corpus-root",
        "--run-id",
        "ltc_legacy_office_seed",
        "--category",
        "policy",
        "--limit",
        "4",
        "--selection-run-id",
        "selection_seed",
        "--selection-root",
        "source-contract/runs/evidence-selection",
        "--converter-command",
        "tools/legacy-office-converter",
        "--dry-run",
    ]);

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::SourceContract { command } = command else {
        panic!("expected episteme source-contract command");
    };
    let EpistemeSourceContractCommand::RunLegacyOfficeConversion(args) = command else {
        panic!("expected episteme source-contract run-legacy-office-conversion command");
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
    assert_eq!(args.run_id, "ltc_legacy_office_seed");
    assert_eq!(args.category.as_deref(), Some("policy"));
    assert_eq!(args.limit, 4);
    assert_eq!(args.selection_run_id.as_deref(), Some("selection_seed"));
    assert_eq!(
        args.converter_command,
        Some(std::path::PathBuf::from("tools/legacy-office-converter"))
    );
    assert!(args.dry_run);
}
