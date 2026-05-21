use super::{
    SEMANTIC_OBJECTS_TABLE_NAME, SEMANTIC_PROJECTION_STATE_TABLE_NAME, TestResult,
    build_semantic_read_model_rows, load_semantic_repository, semantic_read_model_catalog, tempdir,
    write_semantic_read_model_fixture,
};

#[test]
fn semantic_read_model_projects_objects_relations_and_projection_state() -> TestResult {
    let temp_dir = tempdir()?;
    let root = temp_dir.path();
    write_semantic_read_model_fixture(root)?;

    let repository = load_semantic_repository(root);
    assert!(
        repository.report.is_success(),
        "semantic fixture should validate: {:?}",
        repository.report.issues
    );
    let rows = build_semantic_read_model_rows(&repository).map_err(std::io::Error::other)?;

    assert_eq!(rows.objects.len(), 2);
    assert_eq!(rows.relations.len(), 1);
    assert_eq!(rows.projection_state.len(), 1);
    assert!(rows.objects.iter().any(|row| {
        row.id == "component.demo"
            && row.read_model_projection_revision == "semantic-read-model-demo"
            && row.read_model_projection_staleness == "stale"
    }));
    assert!(rows.relations.iter().any(|row| {
        row.source == "component.demo" && row.kind == "validates" && row.target == "task.demo"
    }));
    Ok(())
}

#[test]
fn semantic_read_model_catalog_reports_tables_columns_and_rows() -> TestResult {
    let temp_dir = tempdir()?;
    let root = temp_dir.path();
    write_semantic_read_model_fixture(root)?;

    let repository = load_semantic_repository(root);
    let catalog = semantic_read_model_catalog(&repository).map_err(std::io::Error::other)?;

    assert!(catalog.advisory);
    assert_eq!(catalog.authority, "repo_native_semantic_artifacts");
    assert_eq!(catalog.table_count, 3);
    assert_eq!(catalog.total_row_count, 4);
    let objects = catalog
        .tables
        .iter()
        .find(|table| table.name == SEMANTIC_OBJECTS_TABLE_NAME)
        .ok_or_else(|| std::io::Error::other("semantic_objects table should be cataloged"))?;
    assert_eq!(objects.row_count, 2);
    assert_eq!(objects.column_count, 18);
    assert!(
        objects
            .columns
            .iter()
            .any(|column| column.name == "id" && column.data_type == "Utf8" && !column.nullable)
    );
    let projection_state = catalog
        .tables
        .iter()
        .find(|table| table.name == SEMANTIC_PROJECTION_STATE_TABLE_NAME)
        .ok_or_else(|| {
            std::io::Error::other("semantic_projection_state table should be cataloged")
        })?;
    assert_eq!(projection_state.row_count, 1);
    assert_eq!(projection_state.column_count, 9);
    Ok(())
}
