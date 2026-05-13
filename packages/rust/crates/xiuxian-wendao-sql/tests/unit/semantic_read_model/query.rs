use super::{
    SEMANTIC_OBJECTS_TABLE_NAME, SEMANTIC_PROJECTION_STATE_TABLE_NAME,
    SEMANTIC_RELATIONS_TABLE_NAME, TestResult, build_semantic_read_model_rows,
    load_semantic_repository, query_semantic_read_model_payload, tempdir,
    validate_semantic_read_model_query_text, write_invalid_semantic_fixture,
    write_semantic_read_model_fixture,
};

#[tokio::test]
async fn semantic_read_model_query_exposes_registered_tables() -> TestResult {
    let temp_dir = tempdir()?;
    let root = temp_dir.path();
    write_semantic_read_model_fixture(root)?;

    let payload = query_semantic_read_model_payload(
        root,
        "select o.id, r.kind, p.projection_revision, p.staleness \
         from semantic_objects o \
         join semantic_relations r on o.id = r.source \
         cross join semantic_projection_state p \
         order by o.id",
    )
    .await
    .map_err(std::io::Error::other)?;

    assert_eq!(
        payload.metadata.registered_tables,
        vec![
            SEMANTIC_OBJECTS_TABLE_NAME.to_string(),
            SEMANTIC_RELATIONS_TABLE_NAME.to_string(),
            SEMANTIC_PROJECTION_STATE_TABLE_NAME.to_string(),
        ]
    );
    assert_eq!(payload.metadata.registered_table_count, 3);
    assert_eq!(
        payload.metadata.local_relation_engine.as_deref(),
        Some("datafusion")
    );
    assert!(
        payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(
                |row| row.get("id").and_then(serde_json::Value::as_str) == Some("component.demo")
                    && row.get("kind").and_then(serde_json::Value::as_str) == Some("validates")
                    && row
                        .get("projection_revision")
                        .and_then(serde_json::Value::as_str)
                        == Some("semantic-read-model-demo")
                    && row.get("staleness").and_then(serde_json::Value::as_str) == Some("stale")
            )
    );
    Ok(())
}

#[test]
fn semantic_read_model_rejects_invalid_repository() -> TestResult {
    let temp_dir = tempdir()?;
    let root = temp_dir.path();
    write_invalid_semantic_fixture(root)?;

    let repository = load_semantic_repository(root);
    assert!(!repository.report.is_success());

    let Err(error) = build_semantic_read_model_rows(&repository) else {
        panic!("invalid semantic repository should not project");
    };
    assert!(error.contains("semantic repository validation failed"));
    Ok(())
}

#[test]
fn semantic_read_model_query_validation_accepts_only_single_read_only_query() {
    assert!(
        validate_semantic_read_model_query_text("select id from semantic_objects").is_ok(),
        "plain select should be accepted"
    );
    assert!(
        validate_semantic_read_model_query_text("").is_err(),
        "blank query should be rejected"
    );
    assert!(
        validate_semantic_read_model_query_text("select id from semantic_objects; select 1")
            .is_err(),
        "multi-statement query should be rejected"
    );
    assert!(
        validate_semantic_read_model_query_text(
            "insert into semantic_objects (id) values ('component.bad')"
        )
        .is_err(),
        "mutation statement should be rejected"
    );
}
