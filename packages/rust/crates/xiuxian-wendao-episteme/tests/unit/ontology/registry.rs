use std::{fs, path::Path};

use xiuxian_wendao_episteme::{
    EpistemeOntologyRegistryError, admit_ontology_registry_snapshot, ontology_registry_path,
};

const REGISTRY_SNAPSHOT: &str = include_str!("../../fixtures/ontology_registry_snapshot.json");

#[test]
fn registry_snapshot_admission_accepts_read_model_input() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = tempfile::tempdir()?;
    write_registry_fixture(temp.path(), REGISTRY_SNAPSHOT)?;

    let input = admit_ontology_registry_snapshot(temp.path())?;

    assert_eq!(
        ontology_registry_path(temp.path()),
        temp.path().join("ontology/registry.json")
    );
    assert_eq!(input.snapshot.ontology, "synthetic");
    assert_eq!(input.report.domains, 2);
    assert_eq!(input.report.rdf_files, 2);
    assert_eq!(input.report.rules, 1);
    assert_eq!(input.report.policies, 1);
    assert_eq!(input.report.dataset_mappings, 1);
    assert_eq!(input.report.rdf_classes, 2);
    assert_eq!(input.report.rdf_object_properties, 1);
    assert_eq!(input.report.api_objects, 2);
    assert_eq!(input.report.api_links, 1);
    assert_eq!(input.report.api_actions, 1);
    assert_eq!(input.report.api_queries, 1);
    assert_eq!(input.report.api_interfaces, 1);
    assert_eq!(input.report.reference_nouns, 2);
    Ok(())
}

#[test]
fn registry_snapshot_admission_rejects_duplicate_domain_ids()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let registry = REGISTRY_SNAPSHOT.replace(
        r#""id": "episteme://synthetic/domain""#,
        r#""id": "episteme://synthetic/core""#,
    );
    write_registry_fixture(temp.path(), &registry)?;

    let Err(error) = admit_ontology_registry_snapshot(temp.path()) else {
        return Err("duplicate registry domain ids should be rejected".into());
    };

    assert!(matches!(
        error,
        EpistemeOntologyRegistryError::InvalidSnapshot(_)
    ));
    assert!(
        error
            .to_string()
            .contains("duplicate ontology registry domain id")
    );
    Ok(())
}

#[test]
fn registry_snapshot_admission_rejects_undeclared_domain_reference()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let registry = REGISTRY_SNAPSHOT.replace(
        r#""domain": "episteme://synthetic/domain",
      "kind": "read_only_sql_validation""#,
        r#""domain": "episteme://synthetic/missing",
      "kind": "read_only_sql_validation""#,
    );
    write_registry_fixture(temp.path(), &registry)?;

    let Err(error) = admit_ontology_registry_snapshot(temp.path()) else {
        return Err("undeclared registry domain references should be rejected".into());
    };

    assert!(matches!(
        error,
        EpistemeOntologyRegistryError::InvalidSnapshot(_)
    ));
    assert!(error.to_string().contains("references undeclared domain"));
    Ok(())
}

#[test]
fn registry_snapshot_admission_rejects_unsafe_artifact_path()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let registry = REGISTRY_SNAPSHOT.replace(
        r#""rdf_files": [
        "10_Domain/ontology.rdf"
      ]"#,
        r#""rdf_files": [
        "../outside.rdf"
      ]"#,
    );
    write_registry_fixture(temp.path(), &registry)?;

    let Err(error) = admit_ontology_registry_snapshot(temp.path()) else {
        return Err("unsafe registry artifact paths should be rejected".into());
    };

    assert!(matches!(
        error,
        EpistemeOntologyRegistryError::InvalidSnapshot(_)
    ));
    assert!(
        error
            .to_string()
            .contains("safe paths relative to ontology")
    );
    Ok(())
}

#[test]
fn registry_snapshot_admission_rejects_missing_api_object_reference()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let registry = REGISTRY_SNAPSHOT.replace(
        r#""to_object_type": "SyntheticRelated""#,
        r#""to_object_type": "MissingObject""#,
    );
    write_registry_fixture(temp.path(), &registry)?;

    let Err(error) = admit_ontology_registry_snapshot(temp.path()) else {
        return Err("missing API object references should be rejected".into());
    };

    assert!(matches!(
        error,
        EpistemeOntologyRegistryError::InvalidSnapshot(_)
    ));
    assert!(
        error
            .to_string()
            .contains("references undeclared object type")
    );
    Ok(())
}

fn write_registry_fixture(root: &Path, registry: &str) -> Result<(), Box<dyn std::error::Error>> {
    for path in [
        "ontology/00_Core",
        "ontology/10_Domain/rules",
        "ontology/10_Domain/policies",
        "ontology/10_Domain/mappings/sql",
    ] {
        fs::create_dir_all(root.join(path))?;
    }

    fs::write(root.join("ontology/manifest.toml"), "schema_version = 1\n")?;
    fs::write(
        root.join("ontology/api_surface.toml"),
        "object_types = []\n",
    )?;
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
        root.join("ontology/10_Domain/mappings/ledger.org"),
        "#+TITLE: Mapping Ledger\n",
    )?;
    for file in [
        "01_object_observations.sql",
        "02_semantic_objects.sql",
        "03_semantic_relations.sql",
    ] {
        fs::write(
            root.join("ontology/10_Domain/mappings/sql").join(file),
            "SELECT 1;",
        )?;
    }
    fs::write(root.join("ontology/registry.json"), registry)?;
    Ok(())
}
