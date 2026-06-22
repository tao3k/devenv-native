use std::fs;

use xiuxian_wendao_episteme::{
    EpistemeOntologyError, ontology_manifest_path, validate_ontology_contract,
};

use super::fixtures::{replace_manifest_fragment, write_valid_ontology_fixture};

#[test]
fn ontology_contract_validation_accepts_declared_source_artifacts()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_valid_ontology_fixture(temp.path())?;

    let report = validate_ontology_contract(temp.path())?;

    assert_eq!(
        ontology_manifest_path(temp.path()),
        temp.path().join("ontology/manifest.toml")
    );
    assert_eq!(report.domain_count, 2);
    assert_eq!(report.rdf_file_count, 2);
    assert_eq!(report.rule_count, 1);
    assert_eq!(report.policy_count, 1);
    assert_eq!(report.dataset_mapping_count, 1);
    assert!(report.api_surface_declared);
    Ok(())
}

#[test]
fn ontology_contract_validation_rejects_duplicate_domain_ids()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_valid_ontology_fixture(temp.path())?;
    replace_manifest_fragment(
        temp.path(),
        r#"id = "episteme://synthetic/domain-two""#,
        r#"id = "episteme://synthetic/domain-one""#,
    )?;

    let Err(error) = validate_ontology_contract(temp.path()) else {
        return Err("duplicate domain ids should be rejected".into());
    };

    assert!(matches!(error, EpistemeOntologyError::InvalidContract(_)));
    assert!(error.to_string().contains("duplicate ontology domain id"));
    Ok(())
}

#[test]
fn ontology_contract_validation_rejects_mutable_source_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_valid_ontology_fixture(temp.path())?;
    replace_manifest_fragment(
        temp.path(),
        "mutation_allowed = false",
        "mutation_allowed = true",
    )?;

    let Err(error) = validate_ontology_contract(temp.path()) else {
        return Err("mutable source contract should be rejected".into());
    };

    assert!(matches!(error, EpistemeOntologyError::InvalidContract(_)));
    assert!(error.to_string().contains("mutation_allowed must be false"));
    Ok(())
}

#[test]
fn ontology_contract_validation_rejects_unsafe_artifact_path()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_valid_ontology_fixture(temp.path())?;
    replace_manifest_fragment(
        temp.path(),
        r#"rdf_files = ["00_Core/ontology.rdf"]"#,
        r#"rdf_files = ["../outside.rdf"]"#,
    )?;

    let Err(error) = validate_ontology_contract(temp.path()) else {
        return Err("unsafe artifact path should be rejected".into());
    };

    assert!(matches!(error, EpistemeOntologyError::InvalidContract(_)));
    assert!(
        error
            .to_string()
            .contains("safe paths relative to ontology")
    );
    Ok(())
}

#[test]
fn ontology_contract_validation_rejects_missing_artifact() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = tempfile::tempdir()?;
    write_valid_ontology_fixture(temp.path())?;
    fs::remove_file(temp.path().join("ontology/10_Domain/rules/01_rule.sql"))?;

    let Err(error) = validate_ontology_contract(temp.path()) else {
        return Err("missing artifact should be rejected".into());
    };

    assert!(matches!(error, EpistemeOntologyError::InvalidContract(_)));
    assert!(error.to_string().contains("does not exist"));
    Ok(())
}
