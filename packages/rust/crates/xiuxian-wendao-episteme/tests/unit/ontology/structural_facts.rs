use std::fs;

use serde_json::Value;
use tempfile::tempdir;
use xiuxian_wendao_episteme::{
    EpistemeOntologyStructuralFactsConfiguredRequest, EpistemeOntologyStructuralFactsRequest,
    EpistemeOntologyStructuralFactsValidationMode, write_episteme_ontology_structural_facts,
    write_episteme_ontology_structural_facts_from_config,
};

use super::fixtures::write_structural_facts_fixture;

#[test]
fn structural_facts_writes_documents_anchors_and_relations()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let root = temp.path().join("episteme");
    let corpus_root = temp.path().join("corpus");
    write_structural_facts_fixture(&root, &corpus_root, "expected")?;

    let request =
        EpistemeOntologyStructuralFactsRequest::new(&root, &corpus_root, "structural_seed")
            .with_validation_mode(EpistemeOntologyStructuralFactsValidationMode::FullHash);
    let report = write_episteme_ontology_structural_facts(&request, root.join("runs/structure"))?;

    assert_eq!(report.file_count, 1);
    assert_eq!(report.document_count, 1);
    assert!(report.anchor_count >= 2);
    assert!(report.relation_count >= 1);
    assert!(!report.safety.extraction_executed);
    assert!(!report.safety.source_mutation_allowed);
    assert!(!report.safety.ontology_truth);
    assert!(report.full_hash_checked);
    assert!(report.structural_facts_json.is_file());
    assert!(report.structural_facts_org.is_file());
    assert!(report.rdf_seed_ttl.is_file());
    assert!(report.read_model_objects_json.is_file());
    assert!(report.read_model_relations_json.is_file());
    assert!(report.read_model_projection_state_json.is_file());
    assert!(report.read_model_objects_parquet.is_file());
    assert!(report.read_model_relations_parquet.is_file());
    assert!(report.read_model_quality_passed);
    assert!(report.read_model_quality_issues.is_empty());
    assert!(report.read_model_object_count >= report.anchor_count + report.document_count);
    assert!(report.read_model_relation_count >= report.relation_count);
    assert_eq!(report.read_model_projection_state_count, 1);
    assert!(
        fs::read_to_string(report.documents_tsv)?
            .contains("structural_facts.document.synthetic.file.one")
    );
    assert!(
        fs::read_to_string(&report.read_model_relations_tsv)?
            .contains("read_model_projection_staleness")
    );
    assert!(
        fs::read_to_string(&report.read_model_relations_json)?
            .contains("readModelProjectionStaleness")
    );
    assert!(fs::read_to_string(report.rdf_seed_ttl)?.contains("wdsf:StructuralObject"));
    let objects: Value =
        serde_json::from_str(&fs::read_to_string(report.read_model_objects_json)?)?;
    let relations: Value =
        serde_json::from_str(&fs::read_to_string(report.read_model_relations_json)?)?;
    let projection_state: Value = serde_json::from_str(&fs::read_to_string(
        report.read_model_projection_state_json,
    )?)?;
    assert!(json_array_len(&objects)? >= 2);
    assert!(json_array_len(&relations)? >= 1);
    assert_eq!(json_array_len(&projection_state)?, 1);

    Ok(())
}

#[test]
fn structural_facts_from_config_uses_episteme_toml_defaults()
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
"#,
    )?;

    let request =
        EpistemeOntologyStructuralFactsConfiguredRequest::new(&root, "configured_structural_seed")
            .with_validation_mode(EpistemeOntologyStructuralFactsValidationMode::FullHash);
    let report = write_episteme_ontology_structural_facts_from_config(&request)?;

    assert_eq!(
        report.run_dir,
        root.join("runs/structure/configured_structural_seed")
    );
    assert_eq!(report.file_count, 1);
    assert!(report.full_hash_checked);
    assert!(report.rdf_seed_ttl.is_file());
    assert!(report.read_model_objects_parquet.is_file());
    assert!(report.read_model_quality_passed);

    Ok(())
}

#[test]
fn structural_facts_rejects_sha256_drift() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let root = temp.path().join("episteme");
    let corpus_root = temp.path().join("corpus");
    write_structural_facts_fixture(&root, &corpus_root, "bad_hash")?;

    let request =
        EpistemeOntologyStructuralFactsRequest::new(&root, &corpus_root, "structural_seed")
            .with_validation_mode(EpistemeOntologyStructuralFactsValidationMode::FullHash);
    let error =
        match write_episteme_ontology_structural_facts(&request, root.join("runs/structure")) {
            Ok(report) => panic!("expected sha256 drift error, got report: {report:?}"),
            Err(error) => error,
        };

    assert!(error.to_string().contains("sha256 drift"));
    Ok(())
}

#[test]
fn structural_facts_rejects_duplicate_file_ids() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    let root = temp.path().join("episteme");
    let corpus_root = temp.path().join("corpus");
    write_structural_facts_fixture(&root, &corpus_root, "duplicate_file_id")?;

    let request =
        EpistemeOntologyStructuralFactsRequest::new(&root, &corpus_root, "structural_seed");
    let error =
        match write_episteme_ontology_structural_facts(&request, root.join("runs/structure")) {
            Ok(report) => panic!("expected duplicate file id error, got report: {report:?}"),
            Err(error) => error,
        };

    assert!(error.to_string().contains("duplicate file_id"));
    Ok(())
}

#[test]
#[ignore = "requires WENDAO_EPISTEME_STRUCTURAL_FACTS_ROOT and WENDAO_EPISTEME_STRUCTURAL_FACTS_CORPUS_ROOT"]
fn structural_facts_accepts_configured_real_extension_pack()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(root) = std::env::var_os("WENDAO_EPISTEME_STRUCTURAL_FACTS_ROOT") else {
        panic!("WENDAO_EPISTEME_STRUCTURAL_FACTS_ROOT is required");
    };
    let Some(corpus_root) = std::env::var_os("WENDAO_EPISTEME_STRUCTURAL_FACTS_CORPUS_ROOT") else {
        panic!("WENDAO_EPISTEME_STRUCTURAL_FACTS_CORPUS_ROOT is required");
    };
    let validation_mode = match std::env::var("WENDAO_EPISTEME_STRUCTURAL_FACTS_VALIDATION_MODE")
        .unwrap_or_else(|_| "metadata-only".to_owned())
        .as_str()
    {
        "metadata-only" => EpistemeOntologyStructuralFactsValidationMode::MetadataOnly,
        "full-hash" => EpistemeOntologyStructuralFactsValidationMode::FullHash,
        value => panic!("unsupported WENDAO_EPISTEME_STRUCTURAL_FACTS_VALIDATION_MODE `{value}`"),
    };
    let temp = tempdir()?;
    let request =
        EpistemeOntologyStructuralFactsRequest::new(root, corpus_root, "real_structural_seed")
            .with_validation_mode(validation_mode);
    let report = write_episteme_ontology_structural_facts(&request, temp.path())?;

    assert!(report.file_count > 0);
    assert_eq!(report.document_count, report.file_count);
    assert!(report.anchor_count >= report.document_count);
    assert!(report.relation_count >= report.document_count);
    assert!(report.read_model_quality_passed);
    assert!(report.read_model_objects_parquet.is_file());
    assert!(report.read_model_relations_parquet.is_file());
    assert!(report.rdf_seed_ttl.is_file());
    eprintln!("{report:#?}");
    Ok(())
}

fn json_array_len(value: &Value) -> Result<usize, Box<dyn std::error::Error>> {
    value
        .as_array()
        .map(std::vec::Vec::len)
        .ok_or_else(|| "expected JSON array".into())
}
