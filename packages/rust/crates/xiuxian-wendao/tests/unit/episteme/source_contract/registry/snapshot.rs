use super::support::{
    BTreeSet, admit_and_materialize_episteme_ontology_registry_snapshot_read_model_seed,
    i64_column, materialize_episteme_ontology_registry_snapshot_read_model_seed,
    registry_snapshot_input, string_column, table, validate_episteme_read_model_relation_endpoints,
    write_registry_snapshot_fixture,
};

#[test]
fn episteme_registry_snapshot_materializes_read_model_seed()
-> Result<(), Box<dyn std::error::Error>> {
    let input = registry_snapshot_input()?;

    let materialization = materialize_episteme_ontology_registry_snapshot_read_model_seed(&input)?;
    validate_episteme_read_model_relation_endpoints(&materialization)?;

    assert!(materialization.source_revision.starts_with("sha256:"));
    assert_eq!(materialization.row_counts(), [15, 18, 1]);

    let objects = table(&materialization, "semantic_objects");
    assert_eq!(
        string_column(objects, "id").value(0),
        "episteme_registry.snapshot:synthetic"
    );
    assert_eq!(
        string_column(objects, "kind").value(3),
        "episteme_registry.rdf_class"
    );

    let relations = table(&materialization, "semantic_relations");
    let relation_kinds = (0..relations.num_rows())
        .map(|index| string_column(relations, "kind").value(index).to_string())
        .collect::<BTreeSet<_>>();
    assert!(relation_kinds.contains("episteme_registry.snapshot.declares_domain"));
    assert!(relation_kinds.contains("episteme_registry.api_link.to_object"));
    assert!(relation_kinds.contains("episteme_registry.api_interface.implemented_by"));

    let projection = table(&materialization, "semantic_projection_state");
    assert_eq!(
        string_column(projection, "projection").value(0),
        "episteme_registry.snapshot_read_model_seed.v1"
    );
    assert_eq!(i64_column(projection, "source_object_count").value(0), 15);

    Ok(())
}

#[test]
fn episteme_registry_snapshot_from_root_is_admitted_and_materialized()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    write_registry_snapshot_fixture(temp.path())?;

    let materialization =
        admit_and_materialize_episteme_ontology_registry_snapshot_read_model_seed(temp.path())?;

    assert!(materialization.source_revision.starts_with("sha256:"));
    assert_eq!(materialization.row_counts(), [15, 18, 1]);

    let projection = table(&materialization, "semantic_projection_state");
    assert_eq!(
        string_column(projection, "projection").value(0),
        "episteme_registry.snapshot_read_model_seed.v1"
    );

    Ok(())
}
