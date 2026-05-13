use super::{
    SemanticSqlGuardStatus, TestResult, run_semantic_sql_projection_freshness_guard, tempdir,
    write_semantic_fresh_projection_fixture, write_semantic_read_model_fixture,
};

#[tokio::test]
async fn semantic_sql_guard_projection_freshness_reports_stale_projection() -> TestResult {
    let temp_dir = tempdir()?;
    let root = temp_dir.path();
    write_semantic_read_model_fixture(root)?;

    let evidence = run_semantic_sql_projection_freshness_guard(root)
        .await
        .map_err(std::io::Error::other)?;

    assert_eq!(evidence.status, SemanticSqlGuardStatus::ReviewRequired);
    assert_eq!(evidence.status.as_str(), "review_required");
    assert_eq!(evidence.failing_row_count, 1);
    assert_eq!(evidence.findings.len(), 1);
    assert_eq!(evidence.findings[0].projection, "llm_compression");
    assert_eq!(evidence.findings[0].staleness, "stale");
    assert_eq!(
        evidence.local_relation_engine.as_deref(),
        Some("datafusion")
    );
    assert!(evidence.query_text.contains("semantic_projection_state"));
    assert!(evidence.message.contains("requires review"));
    Ok(())
}

#[tokio::test]
async fn semantic_sql_guard_projection_freshness_passes_with_fresh_projection_rows() -> TestResult {
    let temp_dir = tempdir()?;
    let root = temp_dir.path();
    write_semantic_fresh_projection_fixture(root)?;

    let evidence = run_semantic_sql_projection_freshness_guard(root)
        .await
        .map_err(std::io::Error::other)?;

    assert_eq!(evidence.status, SemanticSqlGuardStatus::Passed);
    assert_eq!(evidence.status.as_str(), "passed");
    assert_eq!(evidence.failing_row_count, 0);
    assert!(evidence.findings.is_empty());
    assert!(evidence.message.contains("passed"));
    Ok(())
}
