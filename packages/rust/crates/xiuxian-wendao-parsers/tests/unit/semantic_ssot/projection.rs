use super::{
    SEMANTIC_PROJECTION_FRESHNESS_POLICY_ID, load_semantic_repository, refresh_projection_as_fresh,
    semantic_change_intent_fixture, semantic_object_fixture, semantic_projection_fixture,
    semantic_projection_freshness_policy_report, semantic_projection_refresh_plan_report, tempdir,
    write_file,
};

#[test]
fn semantic_projection_freshness_policy_reports_required_refresh_targets() {
    let temp = tempdir().unwrap_or_else(|error| panic!("tempdir should exist: {error}"));
    write_file(
        temp.path().join("objects/component/test.md"),
        semantic_object_fixture(
            "component.test",
            "component",
            "Component Test",
            "active",
            "  - kind: validates\n    target: invariant.test\n",
        ),
    );
    write_file(
        temp.path().join("objects/invariant/test.md"),
        semantic_object_fixture(
            "invariant.test",
            "invariant",
            "Invariant Test",
            "active",
            "",
        ),
    );
    write_file(
        temp.path().join("projections/llm-compression.md"),
        semantic_projection_fixture(&["component.test", "invariant.test"], "outdated", "stale"),
    );
    write_file(
        temp.path().join("change-intents/semantic-pilot.md"),
        semantic_change_intent_fixture(
            "component.test",
            "invariant.test",
            "component.test",
            "invariant.test",
            "llm_compression",
        ),
    );

    let repository = load_semantic_repository(temp.path());
    assert!(
        repository.report.is_success(),
        "stale projection should remain a valid advisory artifact: {:?}",
        repository.report.issues
    );
    let report = semantic_projection_freshness_policy_report(&repository);
    assert_eq!(report.policy_id, SEMANTIC_PROJECTION_FRESHNESS_POLICY_ID);
    assert_eq!(report.status, "review_required");
    assert_eq!(report.failing_projection_count, 1);
    assert_eq!(report.projections[0].projection, "llm_compression");
    assert_eq!(report.projections[0].reason, "stale");
    assert_eq!(
        report.projections[0].source_path.as_deref(),
        Some("projections/llm-compression.md")
    );

    refresh_projection_as_fresh(temp.path(), &["component.test", "invariant.test"]);
    let repository = load_semantic_repository(temp.path());
    let report = semantic_projection_freshness_policy_report(&repository);
    assert_eq!(report.status, "passed");
    assert_eq!(report.failing_projection_count, 0);
    assert!(report.projections.is_empty());
}

#[test]
fn semantic_projection_refresh_plan_reports_refreshable_projection_metadata() {
    let temp = tempdir().unwrap_or_else(|error| panic!("tempdir should exist: {error}"));
    write_file(
        temp.path().join("objects/component/test.md"),
        semantic_object_fixture(
            "component.test",
            "component",
            "Component Test",
            "active",
            "",
        ),
    );
    write_file(
        temp.path().join("projections/llm-compression.md"),
        semantic_projection_fixture(&["component.test"], "outdated", "stale"),
    );

    let repository = load_semantic_repository(temp.path());
    assert!(
        repository.report.is_success(),
        "stale projection should remain a valid advisory artifact: {:?}",
        repository.report.issues
    );
    let report = semantic_projection_refresh_plan_report(&repository);
    assert_eq!(report.status, "refresh_required");
    assert_eq!(report.refreshable_projection_count, 1);
    assert_eq!(report.projections[0].projection, "llm_compression");
    assert_eq!(report.projections[0].action, "refresh_source_revision");
    assert_eq!(report.projections[0].reason, "stale");
    assert_eq!(
        report.projections[0].source_path.as_deref(),
        Some("projections/llm-compression.md")
    );

    refresh_projection_as_fresh(temp.path(), &["component.test"]);
    let repository = load_semantic_repository(temp.path());
    let report = semantic_projection_refresh_plan_report(&repository);
    assert_eq!(report.status, "up_to_date");
    assert_eq!(report.refreshable_projection_count, 0);
    assert!(report.projections.is_empty());
}
