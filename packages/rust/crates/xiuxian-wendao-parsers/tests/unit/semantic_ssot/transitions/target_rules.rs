use super::{
    SemanticChangeIntentFixture, load_semantic_repository,
    semantic_change_intent_fixture_with_lifecycle_targets,
    semantic_change_intent_fixture_with_status_transitions, semantic_object_fixture,
    semantic_projection_fixture, tempdir, write_file,
};

#[test]
fn semantic_repository_reports_promotion_transition_missing_target() {
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
    write_file(
        temp.path().join("change-intents/semantic-pilot.md"),
        semantic_change_intent_fixture_with_status_transitions(
            "task.accepted",
            "invariant.test",
            "task.accepted",
            "invariant.test",
            "llm_compression",
            &[],
            &[("task.accepted", "candidate", "active")],
        ),
    );

    let repository = load_semantic_repository(temp.path());
    let messages = repository
        .report
        .issues
        .iter()
        .map(|issue| issue.message.as_str())
        .collect::<Vec<_>>();

    assert!(
        messages
            .iter()
            .any(|message| message.contains("must be listed in promotion_targets")),
        "missing promotion target should be reported: {messages:?}"
    );
}

#[test]
fn semantic_repository_reports_promotion_target_without_transition() {
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
    write_file(
        temp.path().join("change-intents/semantic-pilot.md"),
        semantic_change_intent_fixture_with_lifecycle_targets(SemanticChangeIntentFixture {
            touched_object: "task.accepted",
            affected_invariant: "invariant.test",
            relation_source: "task.accepted",
            relation_target: "invariant.test",
            projection: "llm_compression",
            candidate_suggestions: &[],
            status_transitions: &[],
            promotion_targets: &["task.accepted"],
            demotion_targets: &[],
        }),
    );

    let repository = load_semantic_repository(temp.path());
    let messages = repository
        .report
        .issues
        .iter()
        .map(|issue| issue.message.as_str())
        .collect::<Vec<_>>();

    assert!(
        messages
            .iter()
            .any(|message| message.contains("must match a candidate to active status transition")),
        "promotion target without transition should be reported: {messages:?}"
    );
}

#[test]
fn semantic_repository_reports_demotion_target_without_transition() {
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
    write_file(
        temp.path().join("change-intents/semantic-pilot.md"),
        semantic_change_intent_fixture_with_lifecycle_targets(SemanticChangeIntentFixture {
            touched_object: "task.retired",
            affected_invariant: "invariant.test",
            relation_source: "task.retired",
            relation_target: "invariant.test",
            projection: "llm_compression",
            candidate_suggestions: &[],
            status_transitions: &[],
            promotion_targets: &[],
            demotion_targets: &["task.retired"],
        }),
    );

    let repository = load_semantic_repository(temp.path());
    let messages = repository
        .report
        .issues
        .iter()
        .map(|issue| issue.message.as_str())
        .collect::<Vec<_>>();

    assert!(
        messages.iter().any(|message| message
            .contains("must match a status transition to deprecated, superseded, or retired")),
        "demotion target without transition should be reported: {messages:?}"
    );
}
