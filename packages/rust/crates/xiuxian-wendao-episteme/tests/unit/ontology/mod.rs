use std::{fs, path::Path};

use xiuxian_wendao_episteme::{
    EpistemeOntologyError, ontology_manifest_path, validate_ontology_contract,
};

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

    let error = validate_ontology_contract(temp.path()).unwrap_err();

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

    let error = validate_ontology_contract(temp.path()).unwrap_err();

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

    let error = validate_ontology_contract(temp.path()).unwrap_err();

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

    let error = validate_ontology_contract(temp.path()).unwrap_err();

    assert!(matches!(error, EpistemeOntologyError::InvalidContract(_)));
    assert!(error.to_string().contains("does not exist"));
    Ok(())
}

fn write_valid_ontology_fixture(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    for path in [
        "ontology/00_Core",
        "ontology/10_Domain/rules",
        "ontology/10_Domain/policies",
        "ontology/10_Domain/mappings",
        "ontology/examples/local_project",
    ] {
        fs::create_dir_all(root.join(path))?;
    }

    fs::write(root.join("ontology/00_Core/ontology.rdf"), "<rdf:RDF />")?;
    fs::write(root.join("ontology/10_Domain/ontology.rdf"), "<rdf:RDF />")?;
    fs::write(
        root.join("ontology/10_Domain/rules/01_rule.sql"),
        "SELECT 1;",
    )?;
    fs::write(
        root.join("ontology/10_Domain/policies/policy.md"),
        "# Policy\n",
    )?;
    fs::write(
        root.join("ontology/10_Domain/mappings/mapping.toml"),
        "schema_version = 1\n",
    )?;
    fs::write(
        root.join("ontology/examples/local_project/ontology.toml"),
        "name = \"example\"\n",
    )?;
    fs::write(
        root.join("ontology/api_surface.toml"),
        "object_types = []\n",
    )?;
    fs::write(root.join("ontology/manifest.toml"), valid_manifest())?;
    Ok(())
}

fn replace_manifest_fragment(
    root: &Path,
    needle: &str,
    replacement: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = root.join("ontology/manifest.toml");
    let manifest = fs::read_to_string(&manifest_path)?;
    assert!(
        manifest.contains(needle),
        "manifest fixture missing replacement needle: {needle}"
    );
    fs::write(manifest_path, manifest.replace(needle, replacement))?;
    Ok(())
}

fn valid_manifest() -> &'static str {
    r#"schema_version = 1
name = "synthetic-ontology"

[boundaries]
owner = "wendao-episteme"
artifact_mode = "source_contract"
runtime_compilation_owner = "xiuxian-wendao"
sql_execution_owner = "xiuxian-wendao"
mutation_allowed = false

[[domains]]
id = "episteme://synthetic/domain-one"
category = "00"
layer = "L0"
name = "Synthetic Domain One"
rdf_files = ["00_Core/ontology.rdf"]
rules = []

[[domains]]
id = "episteme://synthetic/domain-two"
category = "10"
layer = "L1"
name = "Synthetic Domain Two"
rdf_files = ["10_Domain/ontology.rdf"]
rules = ["10_Domain/rules/01_rule.sql"]
policies = ["10_Domain/policies/policy.md"]
dataset_mappings = ["10_Domain/mappings/mapping.toml"]

[extension_contract]
example = "examples/local_project/ontology.toml"
extends_field = "ontology.metadata.extends"
namespace_field = "ontology.metadata.namespace"
allowed_sections = ["ontology.metadata", "entity"]
rule_mount = "mount_rules_for_extends_only"

[api_surface]
file = "api_surface.toml"
compatibility = "semantic_api_compatibility"
reference_nouns = ["Ontology", "OntologyObject"]
"#
}
