use super::{
    SEMANTIC_OBJECTS_TABLE_NAME, SEMANTIC_PROJECTION_STATE_TABLE_NAME,
    SEMANTIC_RELATIONS_TABLE_NAME, TestResult, build_semantic_read_model_record_batches,
    build_semantic_read_model_rows, load_semantic_repository,
    semantic_read_model_record_batches_from_rows, tempdir, write_semantic_read_model_fixture,
};

#[test]
fn semantic_read_model_record_batches_match_public_table_contract() -> TestResult {
    let temp = tempdir()?;
    let root = temp.path();
    write_semantic_read_model_fixture(root)?;
    let repository = load_semantic_repository(root);

    let batches =
        build_semantic_read_model_record_batches(&repository).map_err(std::io::Error::other)?;

    assert_eq!(batches.objects.num_rows(), 2);
    assert_eq!(batches.relations.num_rows(), 1);
    assert_eq!(batches.projection_state.num_rows(), 1);
    assert_eq!(batches.objects.schema().field(0).name(), "id");
    assert_eq!(batches.relations.schema().field(0).name(), "source");
    assert_eq!(
        batches.projection_state.schema().field(0).name(),
        "projection"
    );
    assert_eq!(
        batches
            .objects
            .schema()
            .metadata()
            .get("wendao.contract.surface")
            .map(String::as_str),
        Some("arrow-record-batch")
    );
    assert_eq!(
        batches
            .objects
            .schema()
            .metadata()
            .get("wendao.table.name")
            .map(String::as_str),
        Some(SEMANTIC_OBJECTS_TABLE_NAME)
    );
    assert_eq!(SEMANTIC_OBJECTS_TABLE_NAME, "semantic_objects");
    assert_eq!(SEMANTIC_RELATIONS_TABLE_NAME, "semantic_relations");
    assert_eq!(
        SEMANTIC_PROJECTION_STATE_TABLE_NAME,
        "semantic_projection_state"
    );

    let rows = build_semantic_read_model_rows(&repository).map_err(std::io::Error::other)?;
    let rebuilt =
        semantic_read_model_record_batches_from_rows(&rows).map_err(std::io::Error::other)?;
    assert_eq!(rebuilt.objects.num_rows(), batches.objects.num_rows());
    assert_eq!(rebuilt.relations.num_rows(), batches.relations.num_rows());
    assert_eq!(
        rebuilt.projection_state.num_rows(),
        batches.projection_state.num_rows()
    );

    Ok(())
}
