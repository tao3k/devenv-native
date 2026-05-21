use super::{
    SEMANTIC_OBJECTS_TABLE_NAME, TestResult, load_semantic_repository,
    semantic_read_model_materialization_plan, semantic_read_model_materialization_preflight,
    semantic_read_model_snapshot, tempdir, write_semantic_read_model_fixture,
};

#[test]
fn semantic_read_model_materialization_plan_reports_ready_and_blocked_states() -> TestResult {
    let temp_dir = tempdir()?;
    let root = temp_dir.path();
    write_semantic_read_model_fixture(root)?;

    let repository = load_semantic_repository(root);
    let snapshot = semantic_read_model_snapshot(&repository).map_err(std::io::Error::other)?;
    let expected_revision = snapshot.snapshot_revision.clone();

    let ready = semantic_read_model_materialization_plan(
        snapshot.clone(),
        Some(expected_revision.as_str()),
    )
    .map_err(std::io::Error::other)?;
    assert_eq!(ready.status.as_str(), "ready");
    assert!(ready.advisory);
    assert_eq!(ready.authority, "repo_native_semantic_artifacts");
    assert_eq!(ready.target_engine, "duckdb");
    assert_eq!(ready.refresh_discipline, "snapshot_swap");
    assert_eq!(
        ready.writeback_policy,
        "read_model_only_no_semantic_writeback"
    );
    assert_eq!(ready.snapshot_matches_expected, Some(true));
    assert!(
        ready
            .required_steps
            .iter()
            .any(|step| step == "check_expected_snapshot_revision")
    );
    let objects = ready
        .tables
        .iter()
        .find(|table| table.name == SEMANTIC_OBJECTS_TABLE_NAME)
        .ok_or_else(|| {
            std::io::Error::other("semantic_objects table should have a materialization plan")
        })?;
    assert_eq!(objects.row_count, 2);
    assert_eq!(
        objects.planned_registration_strategy,
        "duckdb_materialized_arrow_staging"
    );
    assert_eq!(objects.planned_materialization_state, "materialized");

    let blocked = semantic_read_model_materialization_plan(
        snapshot,
        Some("blake3:0000000000000000000000000000000000000000000000000000000000000000"),
    )
    .map_err(std::io::Error::other)?;
    assert_eq!(blocked.status.as_str(), "blocked");
    assert_eq!(blocked.snapshot_matches_expected, Some(false));
    Ok(())
}

#[tokio::test]
async fn semantic_read_model_materialization_preflight_registers_and_smokes_tables() -> TestResult {
    let temp_dir = tempdir()?;
    let root = temp_dir.path();
    write_semantic_read_model_fixture(root)?;

    let repository = load_semantic_repository(root);
    let snapshot = semantic_read_model_snapshot(&repository).map_err(std::io::Error::other)?;
    let expected_revision = snapshot.snapshot_revision.clone();

    let ready = semantic_read_model_materialization_preflight(
        &repository,
        Some(expected_revision.as_str()),
    )
    .await
    .map_err(std::io::Error::other)?;
    assert_eq!(ready.plan.status.as_str(), "ready");
    assert_eq!(ready.plan.target_engine, "duckdb");
    assert_eq!(ready.plan.snapshot_matches_expected, Some(true));
    let execution = ready
        .execution
        .ok_or_else(|| std::io::Error::other("ready preflight should execute"))?;
    assert_eq!(execution.execution_engine, "duckdb");
    assert_eq!(execution.registered_table_count, 3);
    assert_eq!(execution.registered_input_batch_count, 3);
    assert_eq!(execution.registered_input_row_count, 4);
    assert_eq!(execution.smoke_result_row_count, 3);
    assert!(
        execution.smoke_query.contains("semantic_projection_state"),
        "smoke query should cover all read-model tables"
    );
    let objects = execution
        .tables
        .iter()
        .find(|table| table.name == SEMANTIC_OBJECTS_TABLE_NAME)
        .ok_or_else(|| std::io::Error::other("semantic_objects should be preflighted"))?;
    assert_eq!(objects.row_count, 2);
    assert_eq!(objects.materialization_state, "materialized");
    assert_eq!(
        objects.registration_strategy,
        "duckdb_materialized_arrow_staging"
    );

    let blocked = semantic_read_model_materialization_preflight(
        &repository,
        Some("blake3:0000000000000000000000000000000000000000000000000000000000000000"),
    )
    .await
    .map_err(std::io::Error::other)?;
    assert_eq!(blocked.plan.status.as_str(), "blocked");
    assert!(blocked.execution.is_none());
    Ok(())
}
