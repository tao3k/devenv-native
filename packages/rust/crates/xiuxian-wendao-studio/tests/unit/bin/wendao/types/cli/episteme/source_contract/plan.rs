use super::{Cli, Command, EpistemeCommand, EpistemeSourceContractCommand, Parser};
#[cfg(feature = "episteme-foyer-artifact-cache")]
use crate::bin_support::wendao::types::EpistemeBootstrapArtifactCacheModeArg;
use crate::bin_support::wendao::types::EpistemeStructuralFactsValidationModeArg;

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
fn parses_episteme_source_contract_bootstrap_pipeline_command() {
    let cli = Cli::parse_from([
        "wendao",
        "episteme",
        "source-contract",
        "bootstrap-pipeline",
        "--episteme-root",
        "source-contract",
        "--episteme-registry-id",
        "configured-source",
        "--corpus-root",
        "corpus-root",
        "--structure-run-root",
        "source-contract/runs/structure",
        "--ontology-generation-run-root",
        "source-contract/runs/ontology-generation",
        "--run-id",
        "bootstrap_seed",
        "--validation-mode",
        "full-hash",
        "--category",
        "policy",
        "--route",
        "document_text_evidence",
        "--reasoning-packet-limit",
        "16",
        "--reasoning-ledger-seed-limit",
        "32",
        "--reasoning-fill-plan-limit",
        "64",
    ]);

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::SourceContract { command } = command else {
        panic!("expected episteme source-contract command");
    };
    let EpistemeSourceContractCommand::BootstrapPipeline(args) = command else {
        panic!("expected bootstrap-pipeline command");
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
    assert_eq!(
        args.structure_run_root,
        Some(std::path::PathBuf::from("source-contract/runs/structure"))
    );
    assert_eq!(
        args.ontology_generation_run_root,
        Some(std::path::PathBuf::from(
            "source-contract/runs/ontology-generation"
        ))
    );
    assert_eq!(args.run_id, "bootstrap_seed");
    assert_eq!(
        args.validation_mode,
        EpistemeStructuralFactsValidationModeArg::FullHash
    );
    assert_eq!(args.category.as_deref(), Some("policy"));
    assert_eq!(args.route.as_deref(), Some("document_text_evidence"));
    assert_eq!(args.reasoning_packet_limit, 16);
    assert_eq!(args.reasoning_ledger_seed_limit, 32);
    assert_eq!(args.reasoning_fill_plan_limit, 64);
}

#[cfg(feature = "episteme-foyer-artifact-cache")]
#[test]
fn parses_episteme_source_contract_bootstrap_pipeline_artifact_cache_command() {
    let cli = Cli::parse_from([
        "wendao",
        "episteme",
        "source-contract",
        "bootstrap-pipeline",
        "--episteme-root",
        "source-contract",
        "--run-id",
        "bootstrap_seed",
        "--artifact-cache-mode",
        "read-through",
        "--artifact-cache-source-digest",
        "source-contract-v1",
        "--artifact-cache-profile-digest",
        "bootstrap-v1",
    ]);

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::SourceContract { command } = command else {
        panic!("expected episteme source-contract command");
    };
    let EpistemeSourceContractCommand::BootstrapPipeline(args) = command else {
        panic!("expected bootstrap-pipeline command");
    };

    assert_eq!(
        args.artifact_cache_mode,
        EpistemeBootstrapArtifactCacheModeArg::ReadThrough
    );
    assert_eq!(
        args.artifact_cache_source_digest.as_deref(),
        Some("source-contract-v1")
    );
    assert_eq!(
        args.artifact_cache_profile_digest.as_deref(),
        Some("bootstrap-v1")
    );
}

#[test]
fn parses_episteme_source_contract_write_structural_facts_command() {
    let cli = Cli::parse_from([
        "wendao",
        "episteme",
        "source-contract",
        "write-structural-facts",
        "--episteme-root",
        "source-contract",
        "--episteme-registry-id",
        "configured-source",
        "--corpus-root",
        "corpus-root",
        "--run-root",
        "cache/structure",
        "--run-id",
        "structural_seed",
        "--validation-mode",
        "full-hash",
    ]);

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::SourceContract { command } = command else {
        panic!("expected episteme source-contract command");
    };
    let EpistemeSourceContractCommand::WriteStructuralFacts(args) = command else {
        panic!("expected write-structural-facts command");
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
    assert_eq!(
        args.run_root,
        Some(std::path::PathBuf::from("cache/structure"))
    );
    assert_eq!(
        args.validation_mode,
        EpistemeStructuralFactsValidationModeArg::FullHash
    );
    assert_eq!(args.run_id, "structural_seed");
}

#[test]
fn parses_episteme_source_contract_write_structural_facts_reasoning_packet_command() {
    let cli = Cli::parse_from([
        "wendao",
        "episteme",
        "source-contract",
        "write-structural-facts-reasoning-packet",
        "--episteme-root",
        "source-contract",
        "--structural-facts-run-id",
        "structural_seed",
        "--run-id",
        "reasoning_seed",
        "--category",
        "service_catalog",
        "--route",
        "legacy_office_document_evidence",
        "--limit",
        "4",
    ]);

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::SourceContract { command } = command else {
        panic!("expected episteme source-contract command");
    };
    let EpistemeSourceContractCommand::WriteStructuralFactsReasoningPacket(args) = command else {
        panic!("expected write-structural-facts-reasoning-packet command");
    };

    assert_eq!(
        args.episteme_root,
        std::path::PathBuf::from("source-contract")
    );
    assert_eq!(args.structural_facts_run_id, "structural_seed");
    assert_eq!(args.run_id, "reasoning_seed");
    assert_eq!(args.category.as_deref(), Some("service_catalog"));
    assert_eq!(
        args.route.as_deref(),
        Some("legacy_office_document_evidence")
    );
    assert_eq!(args.limit, 4);
}

#[test]
fn parses_episteme_source_contract_write_qianji_schedule_plan_prompt_audit_command() {
    let cli = Cli::parse_from([
        "wendao",
        "episteme",
        "source-contract",
        "write-structural-facts-reasoning-qianji-schedule-plan",
        "--episteme-root",
        "source-contract",
        "--fill-plan-run-id",
        "reasoning_fill_plan",
        "--run-id",
        "qianji_schedule_plan",
        "--qianji-run-id",
        "episteme.ontology.reasoning.test",
        "--limit",
        "1",
        "--target-ledger-field-group",
        "service_catalog_review",
        "--evidence-target-intent",
        "service_catalog_extraction",
        "--reasoning-context-shard-mode",
        "service-catalog-table-rows",
        "--reasoning-context-shard-row-limit",
        "2",
        "--evidence-extraction-run-root",
        "source-contract/runs/extraction",
        "--evidence-extraction-run-id",
        "ltc_docling_real_probe",
        "--openai-compatible-model",
        "deepseek/deepseek-v4-pro",
        "--openai-compatible-max-tokens",
        "768",
    ]);

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::SourceContract { command } = command else {
        panic!("expected episteme source-contract command");
    };
    let EpistemeSourceContractCommand::WriteStructuralFactsReasoningQianjiSchedulePlan(args) =
        command
    else {
        panic!("expected qianji schedule-plan command");
    };

    assert_eq!(
        args.episteme_root,
        std::path::PathBuf::from("source-contract")
    );
    assert_eq!(args.fill_plan_run_id, "reasoning_fill_plan");
    assert_eq!(args.run_id, "qianji_schedule_plan");
    assert_eq!(
        args.qianji_run_id.as_deref(),
        Some("episteme.ontology.reasoning.test")
    );
    assert_eq!(args.limit, 1);
    assert_eq!(
        args.target_ledger_field_group.as_deref(),
        Some("service_catalog_review")
    );
    assert_eq!(
        args.evidence_target_intent.as_deref(),
        Some("service_catalog_extraction")
    );
    assert_eq!(
        args.reasoning_context_shard_mode,
        "service-catalog-table-rows"
    );
    assert_eq!(args.reasoning_context_shard_row_limit, 2);
    assert_eq!(
        args.evidence_extraction_run_root,
        Some(std::path::PathBuf::from("source-contract/runs/extraction"))
    );
    assert_eq!(
        args.evidence_extraction_run_ids,
        vec!["ltc_docling_real_probe".to_string()]
    );
    assert_eq!(
        args.openai_compatible_model.as_deref(),
        Some("deepseek/deepseek-v4-pro")
    );
    assert_eq!(args.openai_compatible_max_tokens, 768);
}

#[test]
fn parses_episteme_source_contract_inspect_ontology_candidates_command() {
    let cli = Cli::parse_from([
        "wendao",
        "episteme",
        "source-contract",
        "inspect-ontology-candidates",
        "--episteme-root",
        "source-contract",
        "--episteme-registry-id",
        "configured-source",
        "--run-root",
        "source-contract/runs/ontology-generation",
        "--run-id",
        "ontology_seed",
    ]);

    let Command::Episteme { command } = cli.command else {
        panic!("expected episteme command");
    };
    let EpistemeCommand::SourceContract { command } = command else {
        panic!("expected episteme source-contract command");
    };
    let EpistemeSourceContractCommand::InspectOntologyCandidates(args) = command else {
        panic!("expected inspect-ontology-candidates command");
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
        args.run_root,
        Some(std::path::PathBuf::from(
            "source-contract/runs/ontology-generation"
        ))
    );
    assert_eq!(args.run_id, "ontology_seed");
}
