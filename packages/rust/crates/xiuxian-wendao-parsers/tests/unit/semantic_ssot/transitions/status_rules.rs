use super::{
    load_semantic_repository, semantic_change_intent_fixture_with_status_transitions,
    semantic_object_fixture, semantic_projection_fixture, tempdir, write_file,
};

#[test]
fn semantic_repository_reports_status_transition_target_mismatch() {
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
            &[("task.accepted", "candidate", "retired")],
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
            .any(|message| message.contains("current status must match transition target")),
        "target mismatch should be reported: {messages:?}"
    );
}

#[test]
fn semantic_repository_reports_status_transition_missing_touched_object() {
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
        temp.path().join("objects/task/accepted.md"),
        semantic_object_fixture("task.accepted", "task", "Accepted Task", "active", ""),
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
        semantic_projection_fixture(
            &["component.test", "invariant.test", "task.accepted"],
            "outdated",
            "stale",
        ),
    );
    write_file(
        temp.path().join("change-intents/semantic-pilot.md"),
        semantic_change_intent_fixture_with_status_transitions(
            "component.test",
            "invariant.test",
            "component.test",
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
            .any(|message| message.contains("must also be listed in touched_objects")),
        "missing touched object should be reported: {messages:?}"
    );
}

#[test]
fn semantic_repository_reports_disallowed_status_transition() {
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
            &[("task.accepted", "retired", "active")],
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
            .any(|message| message.contains("is not allowed")),
        "disallowed transition should be reported: {messages:?}"
    );
}
