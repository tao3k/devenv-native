use super::{
    SemanticChangeIntentFixture, load_semantic_repository, refresh_projection_as_fresh,
    semantic_change_intent_fixture_with_lifecycle_targets, semantic_object_fixture,
    semantic_projection_fixture, tempdir, write_file,
};

#[test]
fn semantic_repository_accepts_status_transition_intent() {
    let temp = tempdir().unwrap_or_else(|error| panic!("tempdir should exist: {error}"));
    write_file(
        temp.path().join("objects/task/accepted.md"),
        semantic_object_fixture(
            "task.accepted",
            "task",
            "Accepted Task",
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
        semantic_projection_fixture(&["task.accepted", "invariant.test"], "outdated", "stale"),
    );
    refresh_projection_as_fresh(temp.path(), &["task.accepted", "invariant.test"]);
    write_file(
        temp.path().join("change-intents/semantic-pilot.md"),
        semantic_change_intent_fixture_with_lifecycle_targets(SemanticChangeIntentFixture {
            touched_object: "task.accepted",
            affected_invariant: "invariant.test",
            relation_source: "task.accepted",
            relation_target: "invariant.test",
            projection: "llm_compression",
            candidate_suggestions: &[],
            status_transitions: &[("task.accepted", "candidate", "active")],
            promotion_targets: &["task.accepted"],
            demotion_targets: &[],
        }),
    );

    let repository = load_semantic_repository(temp.path());

    assert!(
        repository.report.is_success(),
        "status transition intent should validate: {:?}",
        repository.report.issues
    );
}

#[test]
fn semantic_repository_accepts_demotion_outcome_target() {
    let temp = tempdir().unwrap_or_else(|error| panic!("tempdir should exist: {error}"));
    write_file(
        temp.path().join("objects/task/retired.md"),
        semantic_object_fixture(
            "task.retired",
            "task",
            "Retired Task",
            "retired",
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
        semantic_projection_fixture(&["task.retired", "invariant.test"], "outdated", "stale"),
    );
    refresh_projection_as_fresh(temp.path(), &["task.retired", "invariant.test"]);
    write_file(
        temp.path().join("change-intents/semantic-pilot.md"),
        semantic_change_intent_fixture_with_lifecycle_targets(SemanticChangeIntentFixture {
            touched_object: "task.retired",
            affected_invariant: "invariant.test",
            relation_source: "task.retired",
            relation_target: "invariant.test",
            projection: "llm_compression",
            candidate_suggestions: &[],
            status_transitions: &[("task.retired", "active", "retired")],
            promotion_targets: &[],
            demotion_targets: &["task.retired"],
        }),
    );

    let repository = load_semantic_repository(temp.path());

    assert!(
        repository.report.is_success(),
        "demotion outcome target should validate: {:?}",
        repository.report.issues
    );
}
