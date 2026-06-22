use super::support::{
    BTreeSet, EpistemeRegistryEntry, Path, i64_column, load_episteme_registry_entries,
    materialize_episteme_registry_reference_graph_read_model_seed, string_column, table,
    validate_episteme_read_model_relation_endpoints, validate_episteme_registry_reference_graph,
    write_registry_manifest,
};

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
id = "episteme://private/extension/domain"
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
id = "episteme://private/extension/domain"
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
id = "episteme://private/extension/domain"
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
