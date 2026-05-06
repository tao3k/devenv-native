use super::{
    load_semantic_repository, semantic_change_intent_fixture, semantic_object_fixture,
    semantic_projection_fixture, tempdir, write_file,
};

#[test]
fn semantic_repository_reports_invalid_change_intent_references() {
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
    write_file(
        temp.path().join("change-intents/semantic-pilot.md"),
        semantic_change_intent_fixture(
            "component.missing",
            "component.test",
            "component.test",
            "component.missing",
            "missing_projection",
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
            .any(|message| message.contains("component.missing")),
        "missing touched object or relation target should be reported: {messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("must reference an invariant object")),
        "non-invariant affected invariant should be reported: {messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("missing_projection")),
        "missing projection should be reported: {messages:?}"
    );
}
