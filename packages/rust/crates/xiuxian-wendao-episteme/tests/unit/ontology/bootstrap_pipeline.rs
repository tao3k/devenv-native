use std::fs;

use tempfile::tempdir;
use xiuxian_wendao_episteme::{
    EpistemeOntologyBootstrapPipelineRequest, EpistemeOntologyStructuralFactsValidationMode,
    run_episteme_ontology_bootstrap_pipeline,
};

use super::fixtures::write_structural_facts_fixture;

#[test]
fn bootstrap_pipeline_runs_deterministic_seed_chain_from_episteme_config()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let root = temp.path().join("episteme");
    let corpus_root = temp.path().join("corpus");
    write_structural_facts_fixture(&root, &corpus_root, "expected")?;
    fs::write(
        root.join("episteme.toml"),
        r#"schema_version = 1

[runtime]
corpus_root = "../corpus"
structure_run_root = "runs/structure"
ontology_generation_run_root = "runs/ontology-generation"
"#,
    )?;

    let request = EpistemeOntologyBootstrapPipelineRequest::new(&root, "bootstrap_seed")
        .with_validation_mode(EpistemeOntologyStructuralFactsValidationMode::FullHash);
    let report = run_episteme_ontology_bootstrap_pipeline(&request)?;

    assert_eq!(report.run_id, "bootstrap_seed");
    assert_eq!(report.episteme_root, root);
    assert_eq!(
        report.structural_facts.run_dir,
        report
            .episteme_root
            .join("runs/structure/bootstrap_seed_structural_facts")
    );
    assert_eq!(
        report.ontology_generation_run_root,
        report.episteme_root.join("runs/ontology-generation")
    );
    assert_eq!(report.structural_facts.file_count, 1);
    assert!(report.structural_facts.full_hash_checked);
    assert!(report.structural_facts.structural_facts_json.is_file());
    assert!(report.structural_facts.rdf_seed_ttl.is_file());
    assert!(report.structural_facts.read_model_objects_parquet.is_file());
    assert!(
        report
            .structural_facts
            .read_model_relations_parquet
            .is_file()
    );
    assert!(report.structural_facts.read_model_quality_passed);

    assert_eq!(report.reasoning_packet.packet_row_count, 1);
    assert!(report.reasoning_packet.reasoning_packet_org.is_file());
    assert!(report.reasoning_packet.reasoning_packet_json.is_file());
    assert_eq!(
        report.reasoning_packet.run_dir,
        report
            .ontology_generation_run_root
            .join("bootstrap_seed_reasoning_packet")
    );

    assert!(report.reasoning_ledger_seed.seed_row_count > 0);
    assert!(
        report
            .reasoning_ledger_seed
            .reasoning_ledger_seed_org
            .is_file()
    );
    assert!(
        report
            .reasoning_ledger_seed
            .reasoning_ledger_seed_json
            .is_file()
    );
    assert_eq!(
        report.reasoning_ledger_seed.run_dir,
        report
            .ontology_generation_run_root
            .join("bootstrap_seed_reasoning_ledger_seed")
    );

    assert_eq!(
        report.reasoning_fill_plan.seed_row_count,
        report.reasoning_ledger_seed.seed_row_count
    );
    assert!(report.reasoning_fill_plan.reasoning_fill_plan_org.is_file());
    assert!(
        report
            .reasoning_fill_plan
            .reasoning_fill_plan_json
            .is_file()
    );
    assert_eq!(
        report.reasoning_fill_plan.run_dir,
        report
            .ontology_generation_run_root
            .join("bootstrap_seed_reasoning_fill_plan")
    );

    assert!(!report.safety.source_text_read());
    assert!(!report.safety.llm_executed());
    assert!(!report.safety.workflow_executed());
    assert!(!report.safety.source_mutation_allowed());
    assert!(!report.safety.rdf_mutation_allowed());
    assert!(!report.safety.ontology_truth());

    Ok(())
}

#[test]
#[ignore = "requires WENDAO_EPISTEME_BOOTSTRAP_ROOT and optional WENDAO_EPISTEME_BOOTSTRAP_* overrides"]
fn bootstrap_pipeline_accepts_configured_real_extension_pack()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(root) = std::env::var_os("WENDAO_EPISTEME_BOOTSTRAP_ROOT") else {
        panic!("WENDAO_EPISTEME_BOOTSTRAP_ROOT is required");
    };
    let validation_mode = match std::env::var("WENDAO_EPISTEME_BOOTSTRAP_VALIDATION_MODE")
        .unwrap_or_else(|_| "metadata-only".to_owned())
        .as_str()
    {
        "metadata-only" => EpistemeOntologyStructuralFactsValidationMode::MetadataOnly,
        "full-hash" => EpistemeOntologyStructuralFactsValidationMode::FullHash,
        value => panic!("unsupported WENDAO_EPISTEME_BOOTSTRAP_VALIDATION_MODE `{value}`"),
    };
    let request =
        EpistemeOntologyBootstrapPipelineRequest::new(root, "real_ontology_bootstrap_seed")
            .with_validation_mode(validation_mode);
    let request =
        if let Some(corpus_root) = std::env::var_os("WENDAO_EPISTEME_BOOTSTRAP_CORPUS_ROOT") {
            request.with_corpus_root(corpus_root)
        } else {
            request
        };
    let request =
        if let Some(run_root) = std::env::var_os("WENDAO_EPISTEME_BOOTSTRAP_STRUCTURE_RUN_ROOT") {
            request.with_structure_run_root(run_root)
        } else {
            request
        };
    let request = if let Some(run_root) =
        std::env::var_os("WENDAO_EPISTEME_BOOTSTRAP_ONTOLOGY_GENERATION_RUN_ROOT")
    {
        request.with_ontology_generation_run_root(run_root)
    } else {
        request
    };
    let report = run_episteme_ontology_bootstrap_pipeline(&request)?;

    assert!(report.structural_facts.file_count > 0);
    assert_eq!(
        report.structural_facts.document_count,
        report.structural_facts.file_count
    );
    assert!(report.structural_facts.anchor_count >= report.structural_facts.document_count);
    assert!(report.structural_facts.relation_count >= report.structural_facts.document_count);
    assert!(report.structural_facts.read_model_quality_passed);
    assert!(report.reasoning_packet.packet_row_count > 0);
    assert!(report.reasoning_ledger_seed.seed_row_count > 0);
    assert!(report.reasoning_fill_plan.fill_item_count > 0);
    assert!(!report.safety.source_text_read());
    assert!(!report.safety.llm_executed());
    assert!(!report.safety.workflow_executed());
    assert!(!report.safety.ontology_truth());
    eprintln!("{report:#?}");
    Ok(())
}
