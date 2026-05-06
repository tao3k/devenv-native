use super::{
    SemanticScopeRequest, load_semantic_repository, refresh_projection_as_fresh,
    semantic_change_intent_fixture_with_candidates, semantic_object_fixture,
    semantic_object_fixture_with_confidence, semantic_projection_fixture, semantic_scope_bundle,
    tempdir, write_file,
};

#[test]
fn semantic_repository_accepts_governed_candidate_object() {
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
        temp.path().join("objects/task/candidate.md"),
        semantic_object_fixture_with_confidence(
            "task.candidate",
            "task",
            "Candidate Task",
            "candidate",
            "llm_suggested",
            "",
        ),
    );
    write_file(
        temp.path().join("projections/llm-compression.md"),
        semantic_projection_fixture(
            &["component.test", "invariant.test", "task.candidate"],
            "outdated",
            "stale",
        ),
    );
    refresh_projection_as_fresh(
        temp.path(),
        &["component.test", "invariant.test", "task.candidate"],
    );
    write_file(
        temp.path().join("change-intents/semantic-pilot.md"),
        semantic_change_intent_fixture_with_candidates(
            "component.test",
            "invariant.test",
            "component.test",
            "invariant.test",
            "llm_compression",
            &["task.candidate"],
        ),
    );

    let repository = load_semantic_repository(temp.path());

    assert!(
        repository.report.is_success(),
        "governed candidate object should validate: {:?}",
        repository.report.issues
    );

    let bundle = semantic_scope_bundle(
        &repository,
        &SemanticScopeRequest {
            task_id: None,
            object_ids: vec!["task.candidate".to_string()],
        },
    );

    assert_eq!(
        bundle
            .change_intents
            .iter()
            .map(|intent| intent.id.as_str())
            .collect::<Vec<_>>(),
        vec!["change.semantic-ssot.test"]
    );
}

#[test]
fn semantic_repository_reports_ungoverned_candidate_object() {
    let temp = tempdir().unwrap_or_else(|error| panic!("tempdir should exist: {error}"));
    write_file(
        temp.path().join("objects/task/candidate.md"),
        semantic_object_fixture_with_confidence(
            "task.candidate",
            "task",
            "Candidate Task",
            "candidate",
            "llm_suggested",
            "",
        ),
    );
    write_file(
        temp.path().join("projections/llm-compression.md"),
        semantic_projection_fixture(&["task.candidate"], "outdated", "stale"),
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
            .any(|message| message.contains("active change intent candidate_suggestions entry")),
        "ungoverned candidate object should be reported: {messages:?}"
    );
}

#[test]
fn semantic_repository_reports_candidate_with_authoritative_confidence() {
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
        temp.path().join("objects/task/candidate.md"),
        semantic_object_fixture("task.candidate", "task", "Candidate Task", "candidate", ""),
    );
    write_file(
        temp.path().join("projections/llm-compression.md"),
        semantic_projection_fixture(
            &["component.test", "invariant.test", "task.candidate"],
            "outdated",
            "stale",
        ),
    );
    write_file(
        temp.path().join("change-intents/semantic-pilot.md"),
        semantic_change_intent_fixture_with_candidates(
            "component.test",
            "invariant.test",
            "component.test",
            "invariant.test",
            "llm_compression",
            &["task.candidate"],
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
            .any(|message| message
                .contains("must use `llm_suggested` confidence source until accepted")),
        "authoritative candidate confidence should be reported: {messages:?}"
    );
}
