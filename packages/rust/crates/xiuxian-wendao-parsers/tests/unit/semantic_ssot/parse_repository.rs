use super::{
    Path, load_semantic_repository, parse_semantic_object, refresh_projection_as_fresh,
    semantic_change_intent_fixture, semantic_object_fixture, semantic_projection_fixture, tempdir,
    write_file,
};

#[test]
fn semantic_parser_accepts_object_frontmatter() {
    let object = parse_semantic_object(
        "objects/component/test.md",
        &semantic_object_fixture(
            "component.test",
            "component",
            "Component Test",
            "active",
            "",
        ),
    )
    .unwrap_or_else(|error| panic!("semantic object should parse: {error}"));

    assert_eq!(object.id, "component.test");
    assert_eq!(object.relations.len(), 0);
    assert_eq!(object.source_path, Path::new("objects/component/test.md"));
}

#[test]
fn semantic_repository_validates_relation_targets_and_projection_sources() {
    let temp = tempdir().unwrap_or_else(|error| panic!("tempdir should exist: {error}"));
    write_file(
        temp.path().join("objects/component/test.md"),
        semantic_object_fixture(
            "component.test",
            "component",
            "Component Test",
            "active",
            "  - kind: depends_on\n    target: decision.missing\n",
        ),
    );
    write_file(
        temp.path().join("projections/llm-compression.md"),
        concat!(
            "---\n",
            "type: semantic_projection\n",
            "projection: llm_compression\n",
            "source_objects:\n",
            "  - component.missing\n",
            "source_revision: stale-fixture\n",
            "projection_revision: test.v1\n",
            "staleness: stale\n",
            "status: active\n",
            "---\n",
            "# Projection\n",
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
            .any(|message| message.contains("decision.missing")),
        "missing relation target should be reported: {messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("component.missing")),
        "missing projection source should be reported: {messages:?}"
    );
}
#[test]
fn semantic_repository_flags_unmarked_stale_projection_source_revision() {
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
        semantic_projection_fixture(&["component.test"], "outdated", "fresh"),
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
            .any(|message| message.contains("source revision is stale")),
        "stale fresh projection should be reported: {messages:?}"
    );
}

#[test]
fn semantic_repository_accepts_explicitly_stale_projection_source_revision() {
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
        "explicit stale projection should validate: {:?}",
        repository.report.issues
    );
}

#[test]
fn semantic_repository_accepts_valid_change_intent() {
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
    refresh_projection_as_fresh(temp.path(), &["component.test", "invariant.test"]);
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
        "valid change intent should pass: {:?}",
        repository.report.issues
    );
    assert_eq!(repository.change_intents.len(), 1);
}
