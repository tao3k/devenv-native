use xiuxian_wendao_episteme::{EpistemeOntologyError, validate_ontology_contract};

use super::fixtures::{replace_manifest_fragment, write_extension_source_contract_fixture};

#[test]
fn ontology_contract_validation_accepts_extension_source_contract_manifest()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_extension_source_contract_fixture(temp.path())?;

    let report = validate_ontology_contract(temp.path())?;

    assert_eq!(report.domain_count, 1);
    assert_eq!(report.rdf_file_count, 1);
    assert_eq!(report.rule_count, 0);
    assert_eq!(report.policy_count, 0);
    assert_eq!(report.dataset_mapping_count, 0);
    Ok(())
}

#[test]
fn ontology_contract_validation_rejects_extension_source_contract_without_extends()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_extension_source_contract_fixture(temp.path())?;
    replace_manifest_fragment(
        temp.path(),
        r#"
[extends]
common_manifest = "episteme://synthetic/healthcare"
common_ontology_iri = "https://wendao.ai/ontology/healthcare"
"#,
        "",
    )?;

    let Err(error) = validate_ontology_contract(temp.path()) else {
        return Err("extension source contract without extends should be rejected".into());
    };

    assert!(matches!(error, EpistemeOntologyError::InvalidContract(_)));
    assert!(error.to_string().contains("must declare [extends]"));
    Ok(())
}

#[test]
fn ontology_contract_validation_rejects_invalid_extension_domain_scheme()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_extension_source_contract_fixture(temp.path())?;
    replace_manifest_fragment(
        temp.path(),
        r#"id = "episteme://synthetic-extension/10_Extension""#,
        r#"id = "private://synthetic/10_Extension""#,
    )?;

    let Err(error) = validate_ontology_contract(temp.path()) else {
        return Err("invalid extension domain scheme should be rejected".into());
    };

    assert!(matches!(error, EpistemeOntologyError::InvalidContract(_)));
    assert!(error.to_string().contains("episteme://"));
    Ok(())
}
