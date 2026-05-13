use super::{
    SEMANTIC_PROJECTION_STATE_TABLE_NAME, SEMANTIC_RELATIONS_TABLE_NAME, TestResult,
    load_semantic_repository, semantic_read_model_snapshot, semantic_read_model_snapshot_check,
    tempdir, write_semantic_read_model_fixture,
};

#[test]
fn semantic_read_model_snapshot_reports_deterministic_revisions() -> TestResult {
    let temp_dir = tempdir()?;
    let root = temp_dir.path();
    write_semantic_read_model_fixture(root)?;

    let repository = load_semantic_repository(root);
    let snapshot = semantic_read_model_snapshot(&repository).map_err(std::io::Error::other)?;
    let repeated_snapshot =
        semantic_read_model_snapshot(&repository).map_err(std::io::Error::other)?;

    assert!(snapshot.advisory);
    assert_eq!(snapshot.authority, "repo_native_semantic_artifacts");
    assert_eq!(snapshot.catalog.table_count, 3);
    assert_eq!(snapshot.catalog.total_row_count, 4);
    assert_eq!(
        snapshot.snapshot_revision,
        repeated_snapshot.snapshot_revision
    );
    assert!(snapshot.snapshot_revision.starts_with("blake3:"));
    assert_eq!(snapshot.snapshot_revision.len(), "blake3:".len() + 64);
    let relations = snapshot
        .tables
        .iter()
        .find(|table| table.name == SEMANTIC_RELATIONS_TABLE_NAME)
        .ok_or_else(|| std::io::Error::other("semantic_relations table should be snapshotted"))?;
    assert_eq!(relations.row_count, 1);
    assert_eq!(relations.column_count, 7);
    assert!(relations.row_revision.starts_with("blake3:"));
    let projection_state = snapshot
        .tables
        .iter()
        .find(|table| table.name == SEMANTIC_PROJECTION_STATE_TABLE_NAME)
        .ok_or_else(|| {
            std::io::Error::other("semantic_projection_state table should be snapshotted")
        })?;
    assert_eq!(projection_state.row_count, 1);
    assert_ne!(relations.row_revision, projection_state.row_revision);
    Ok(())
}

#[test]
fn semantic_read_model_snapshot_check_reports_match_and_mismatch() -> TestResult {
    let temp_dir = tempdir()?;
    let root = temp_dir.path();
    write_semantic_read_model_fixture(root)?;

    let repository = load_semantic_repository(root);
    let snapshot = semantic_read_model_snapshot(&repository).map_err(std::io::Error::other)?;
    let expected_revision = snapshot.snapshot_revision.clone();

    let matched = semantic_read_model_snapshot_check(snapshot.clone(), expected_revision.as_str())
        .map_err(std::io::Error::other)?;
    assert!(matched.matches);
    assert_eq!(matched.expected_snapshot_revision, expected_revision);
    assert_eq!(
        matched.current_snapshot_revision,
        matched.current_snapshot.snapshot_revision
    );

    let mismatched = semantic_read_model_snapshot_check(
        snapshot,
        "blake3:0000000000000000000000000000000000000000000000000000000000000000",
    )
    .map_err(std::io::Error::other)?;
    assert!(!mismatched.matches);
    assert_eq!(
        mismatched.expected_snapshot_revision,
        "blake3:0000000000000000000000000000000000000000000000000000000000000000"
    );

    let Err(invalid) =
        semantic_read_model_snapshot_check(mismatched.current_snapshot, "not-a-revision")
    else {
        return Err(
            std::io::Error::other("non-blake3 expected revision should be rejected").into(),
        );
    };
    assert!(
        invalid.contains("blake3"),
        "invalid revision should explain the scheme: {invalid}"
    );
    Ok(())
}
