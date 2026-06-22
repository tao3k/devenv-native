use super::{Cli, Command, EpistemeCommand, EpistemeSourceContractCommand, Parser};

#[test]
fn parses_episteme_source_contract_write_ontology_source_patch_preflight_command() {
    let cli = source_patch_cli("write-ontology-source-patch-preflight");

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::SourceContract { command } = command else {
        panic!("expected episteme source-contract command");
    };
    let EpistemeSourceContractCommand::WriteOntologySourcePatchPreflight(args) = command else {
        panic!("expected episteme source-contract write-ontology-source-patch-preflight command");
    };
    assert_source_patch_args(
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
        args.run_root.as_ref(),
        args.run_id.as_str(),
    );
}

#[test]
fn parses_episteme_source_contract_write_ontology_source_patch_draft_command() {
    let cli = source_patch_cli("write-ontology-source-patch-draft");

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::SourceContract { command } = command else {
        panic!("expected episteme source-contract command");
    };
    let EpistemeSourceContractCommand::WriteOntologySourcePatchDraft(args) = command else {
        panic!("expected episteme source-contract write-ontology-source-patch-draft command");
    };
    assert_source_patch_args(
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
        args.run_root.as_ref(),
        args.run_id.as_str(),
    );
}

#[test]
fn parses_episteme_source_contract_write_ontology_source_patch_apply_plan_command() {
    let cli = source_patch_cli("write-ontology-source-patch-apply-plan");

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::SourceContract { command } = command else {
        panic!("expected episteme source-contract command");
    };
    let EpistemeSourceContractCommand::WriteOntologySourcePatchApplyPlan(args) = command else {
        panic!("expected episteme source-contract write-ontology-source-patch-apply-plan command");
    };
    assert_source_patch_args(
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
        args.run_root.as_ref(),
        args.run_id.as_str(),
    );
}

#[test]
fn parses_episteme_source_contract_write_ontology_source_patch_review_packet_command() {
    let cli = source_patch_cli("write-ontology-source-patch-review-packet");

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::SourceContract { command } = command else {
        panic!("expected episteme source-contract command");
    };
    let EpistemeSourceContractCommand::WriteOntologySourcePatchReviewPacket(args) = command else {
        panic!(
            "expected episteme source-contract write-ontology-source-patch-review-packet command"
        );
    };
    assert_source_patch_args(
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
        args.run_root.as_ref(),
        args.run_id.as_str(),
    );
}

#[test]
fn parses_episteme_source_contract_write_ontology_source_patch_apply_preview_command() {
    let cli = Cli::parse_from([
        "wendao",
        "episteme",
        "source-contract",
        "write-ontology-source-patch-apply-preview",
        "--episteme-root",
        "source-contract",
        "--episteme-registry-id",
        "configured-source",
        "--run-root",
        "source-contract/runs/source-patch-preflight",
        "--run-id",
        "ltc_preflight_seed",
        "--expected-apply-plan-tsv-sha256",
        "abc123",
    ]);

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::SourceContract { command } = command else {
        panic!("expected episteme source-contract command");
    };
    let EpistemeSourceContractCommand::WriteOntologySourcePatchApplyPreview(args) = command else {
        panic!(
            "expected episteme source-contract write-ontology-source-patch-apply-preview command"
        );
    };
    assert_source_patch_args(
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
        args.run_root.as_ref(),
        args.run_id.as_str(),
    );
    assert_eq!(args.expected_apply_plan_tsv_sha256, "abc123");
}

#[test]
fn parses_episteme_source_contract_write_ontology_source_patch_semantic_preview_command() {
    let cli = source_patch_cli("write-ontology-source-patch-semantic-preview");

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::SourceContract { command } = command else {
        panic!("expected episteme source-contract command");
    };
    let EpistemeSourceContractCommand::WriteOntologySourcePatchSemanticPreview(args) = command
    else {
        panic!(
            "expected episteme source-contract write-ontology-source-patch-semantic-preview command"
        );
    };
    assert_source_patch_args(
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
        args.run_root.as_ref(),
        args.run_id.as_str(),
    );
}

#[test]
fn parses_episteme_source_contract_write_ontology_source_patch_rdf_read_model_command() {
    let cli = source_patch_cli("write-ontology-source-patch-rdf-read-model");

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::SourceContract { command } = command else {
        panic!("expected episteme source-contract command");
    };
    let EpistemeSourceContractCommand::WriteOntologySourcePatchRdfReadModel(args) = command else {
        panic!(
            "expected episteme source-contract write-ontology-source-patch-rdf-read-model command"
        );
    };
    assert_source_patch_args(
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
        args.run_root.as_ref(),
        args.run_id.as_str(),
    );
}

#[test]
fn parses_episteme_source_contract_apply_ontology_source_patch_command() {
    let cli = Cli::parse_from([
        "wendao",
        "episteme",
        "source-contract",
        "apply-ontology-source-patch",
        "--episteme-root",
        "source-contract",
        "--episteme-registry-id",
        "configured-source",
        "--run-root",
        "source-contract/runs/source-patch-preflight",
        "--run-id",
        "ltc_preflight_seed",
        "--expected-apply-plan-tsv-sha256",
        "abc123",
        "--allow-source-mutation",
    ]);

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::SourceContract { command } = command else {
        panic!("expected episteme source-contract command");
    };
    let EpistemeSourceContractCommand::ApplyOntologySourcePatch(args) = command else {
        panic!("expected episteme source-contract apply-ontology-source-patch command");
    };
    assert_source_patch_args(
        &args.episteme_root,
        args.episteme_registry_id.as_deref(),
        args.run_root.as_ref(),
        args.run_id.as_str(),
    );
    assert_eq!(args.expected_apply_plan_tsv_sha256, "abc123");
    assert!(args.allow_source_mutation);
}

fn source_patch_cli(command: &'static str) -> Cli {
    Cli::parse_from([
        "wendao",
        "episteme",
        "source-contract",
        command,
        "--episteme-root",
        "source-contract",
        "--episteme-registry-id",
        "configured-source",
        "--run-root",
        "source-contract/runs/source-patch-preflight",
        "--run-id",
        "ltc_preflight_seed",
    ])
}

fn assert_source_patch_args(
    episteme_root: &std::path::PathBuf,
    episteme_registry_id: Option<&str>,
    run_root: Option<&std::path::PathBuf>,
    run_id: &str,
) {
    assert_eq!(episteme_root, &std::path::PathBuf::from("source-contract"));
    assert_eq!(episteme_registry_id, Some("configured-source"));
    assert_eq!(
        run_root,
        Some(&std::path::PathBuf::from(
            "source-contract/runs/source-patch-preflight"
        ))
    );
    assert_eq!(run_id, "ltc_preflight_seed");
}
