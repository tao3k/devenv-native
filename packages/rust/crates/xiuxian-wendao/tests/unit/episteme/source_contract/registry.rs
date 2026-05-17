use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use super::support::{
    EpistemeFixture, cleanup_managed_git_entry, i64_column, init_git_repository, string_column,
    table, write_registry_manifest,
};
use xiuxian_wendao::episteme::{
    EpistemeRegistryEntry, LoadedEpistemeSourceKind, load_episteme_registry_entries,
    materialize_episteme_registry_reference_graph_read_model_seed,
    validate_episteme_read_model_relation_endpoints, validate_episteme_registry_reference_graph,
};

#[test]
fn episteme_registry_loads_local_path_entry() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = EpistemeFixture::new()?;
    fixture.write_contract()?;

    let receipt = load_episteme_registry_entries(
        &[EpistemeRegistryEntry::local(
            "source_contract",
            fixture.episteme_root.clone(),
        )],
        Path::new("."),
    )?;

    assert_eq!(receipt.loaded_count, 1);
    assert_eq!(receipt.entries[0].id, "source_contract");
    assert_eq!(
        receipt.entries[0].source_kind,
        LoadedEpistemeSourceKind::Local
    );
    assert_eq!(receipt.entries[0].episteme_root, fixture.episteme_root);
    assert_eq!(receipt.entries[0].subdir, ".");
    assert!(receipt.entries[0].resolved_revision.is_none());
    Ok(())
}

#[test]
fn episteme_registry_filters_disabled_entries() -> Result<(), Box<dyn std::error::Error>> {
    let receipt = load_episteme_registry_entries(
        &[EpistemeRegistryEntry {
            id: "disabled_entry".to_string(),
            path: None,
            url: None,
            enabled: false,
            subdir: PathBuf::from("."),
        }],
        Path::new("."),
    )?;

    assert_eq!(receipt.loaded_count, 0);
    assert!(receipt.entries.is_empty());
    Ok(())
}

#[test]
fn episteme_registry_rejects_mixed_path_and_url() {
    let result = load_episteme_registry_entries(
        &[EpistemeRegistryEntry {
            id: "mixed".to_string(),
            path: Some(PathBuf::from(".")),
            url: Some("https://github.com/example/example-episteme.git".to_string()),
            enabled: true,
            subdir: PathBuf::from("."),
        }],
        Path::new("."),
    );
    let Err(error) = result else {
        panic!("mixed path/url entry should fail");
    };

    assert!(error.to_string().contains("exactly one of `path` or `url`"));
}

#[test]
fn episteme_registry_rejects_unsafe_subdir() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = EpistemeFixture::new()?;
    fixture.write_contract()?;

    let result = load_episteme_registry_entries(
        &[EpistemeRegistryEntry::local("unsafe", fixture.episteme_root).with_subdir("../escape")],
        Path::new("."),
    );
    let Err(error) = result else {
        panic!("unsafe subdir should fail");
    };

    assert!(error.to_string().contains("unsafe subdir"));
    Ok(())
}

#[test]
fn episteme_registry_materializes_git_url_entry() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = EpistemeFixture::new()?;
    fixture.write_contract()?;
    init_git_repository(fixture.episteme_root.as_path())?;
    let url = fixture.episteme_root.display().to_string();

    let receipt = load_episteme_registry_entries(
        &[EpistemeRegistryEntry::git("remote_source", url.clone())],
        Path::new("."),
    )?;

    assert_eq!(receipt.loaded_count, 1);
    assert_eq!(receipt.entries[0].id, "remote_source");
    assert_eq!(
        receipt.entries[0].source_kind,
        LoadedEpistemeSourceKind::Git
    );
    assert_eq!(receipt.entries[0].url.as_deref(), Some(url.as_str()));
    assert!(
        receipt.entries[0]
            .episteme_root
            .join("ontology/manifest.toml")
            .is_file()
    );
    assert!(receipt.entries[0].resolved_revision.is_some());

    cleanup_managed_git_entry("remote_source", url.as_str())?;
    Ok(())
}

#[test]
fn episteme_registry_reference_graph_accepts_satisfied_extension_target()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let common_root = temp.path().join("common-episteme");
    let extension_root = temp.path().join("extension-episteme");
    write_registry_manifest(
        common_root.as_path(),
        r#"schema_version = 1
name = "common-episteme"

[[domains]]
id = "episteme://common/domain"
"#,
    )?;
    write_registry_manifest(
        extension_root.as_path(),
        r#"schema_version = 1
name = "extension-episteme"

[extends]
manifest = "episteme://common/domain"

[[domains]]
id = "private://extension/domain"
"#,
    )?;

    let receipt = load_episteme_registry_entries(
        &[
            EpistemeRegistryEntry::local("common", common_root),
            EpistemeRegistryEntry::local("extension", extension_root),
        ],
        Path::new("."),
    )?;
    let graph = validate_episteme_registry_reference_graph(&receipt)?;

    assert_eq!(graph.entry_count, 2);
    assert_eq!(graph.domain_count, 2);
    assert_eq!(graph.reference_links.len(), 1);
    assert_eq!(graph.reference_links[0].source_registry, "extension");
    assert_eq!(
        graph.reference_links[0].target_domain,
        "episteme://common/domain"
    );
    assert_eq!(graph.reference_links[0].target_registry, "common");
    Ok(())
}

#[test]
fn episteme_registry_reference_graph_rejects_missing_extension_target()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let extension_root = temp.path().join("extension-episteme");
    write_registry_manifest(
        extension_root.as_path(),
        r#"schema_version = 1
name = "extension-episteme"

[extends]
manifest = "episteme://missing/domain"

[[domains]]
id = "private://extension/domain"
"#,
    )?;

    let receipt = load_episteme_registry_entries(
        &[EpistemeRegistryEntry::local("extension", extension_root)],
        Path::new("."),
    )?;
    let Err(error) = validate_episteme_registry_reference_graph(&receipt) else {
        panic!("missing extension target should fail");
    };

    assert!(error.to_string().contains("episteme://missing/domain"));
    assert!(error.to_string().contains("no loaded registry owns it"));
    Ok(())
}

#[test]
fn episteme_registry_reference_graph_rejects_duplicate_domain_ids()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let first_root = temp.path().join("first-episteme");
    let second_root = temp.path().join("second-episteme");
    let manifest = r#"schema_version = 1
name = "duplicate-domain-episteme"

[[domains]]
id = "episteme://duplicate/domain"
"#;
    write_registry_manifest(first_root.as_path(), manifest)?;
    write_registry_manifest(second_root.as_path(), manifest)?;

    let receipt = load_episteme_registry_entries(
        &[
            EpistemeRegistryEntry::local("first", first_root),
            EpistemeRegistryEntry::local("second", second_root),
        ],
        Path::new("."),
    )?;
    let Err(error) = validate_episteme_registry_reference_graph(&receipt) else {
        panic!("duplicate domain ids should fail");
    };

    assert!(error.to_string().contains("episteme://duplicate/domain"));
    assert!(error.to_string().contains("first"));
    assert!(error.to_string().contains("second"));
    Ok(())
}

#[test]
fn episteme_registry_reference_graph_materializes_read_model_seed()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let common_root = temp.path().join("common-episteme");
    let extension_root = temp.path().join("extension-episteme");
    write_registry_manifest(
        common_root.as_path(),
        r#"schema_version = 1
name = "common-episteme"

[[domains]]
id = "episteme://common/domain"
"#,
    )?;
    write_registry_manifest(
        extension_root.as_path(),
        r#"schema_version = 1
name = "extension-episteme"

[extends]
manifest = "episteme://common/domain"

[[domains]]
id = "private://extension/domain"
"#,
    )?;

    let receipt = load_episteme_registry_entries(
        &[
            EpistemeRegistryEntry::local("common", common_root),
            EpistemeRegistryEntry::local("extension", extension_root),
        ],
        Path::new("."),
    )?;
    let graph = validate_episteme_registry_reference_graph(&receipt)?;
    let materialization = materialize_episteme_registry_reference_graph_read_model_seed(&graph)?;
    validate_episteme_read_model_relation_endpoints(&materialization)?;

    assert!(materialization.source_revision.starts_with("sha256:"));
    assert_eq!(materialization.row_counts(), [4, 3, 1]);

    let objects = table(&materialization, "semantic_objects");
    let object_ids = string_column(objects, "id");
    assert_eq!(object_ids.value(0), "episteme_registry.entry:common");
    assert_eq!(
        string_column(objects, "kind").value(1),
        "episteme_registry.domain"
    );

    let relations = table(&materialization, "semantic_relations");
    let relation_kinds = (0..relations.num_rows())
        .map(|index| string_column(relations, "kind").value(index).to_string())
        .collect::<BTreeSet<_>>();
    assert!(relation_kinds.contains("episteme_registry.loaded_entry.owns_domain"));
    assert!(relation_kinds.contains("episteme_registry.loaded_entry.extends_domain"));

    let projection = table(&materialization, "semantic_projection_state");
    assert_eq!(
        string_column(projection, "projection").value(0),
        "episteme_registry.reference_graph_read_model_seed.v1"
    );
    assert_eq!(i64_column(projection, "source_object_count").value(0), 4);

    Ok(())
}
