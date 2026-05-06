use super::{
    SEMANTIC_PROJECTION_FRESHNESS_POLICY_ID, SemanticScopeRequest, load_semantic_repository,
    parse_semantic_scope_metadata_envelope_json, refresh_projection_as_fresh,
    semantic_change_intent_fixture, semantic_object_fixture, semantic_projection_fixture,
    semantic_projection_freshness_policy_report, semantic_scope_bundle,
    semantic_scope_metadata_envelope, semantic_scope_metadata_envelope_to_vec, tempdir, write_file,
};

#[test]
fn semantic_scope_bundle_includes_active_relation_neighborhood() {
    let temp = tempdir().unwrap_or_else(|error| panic!("tempdir should exist: {error}"));
    write_file(
        temp.path().join("objects/task/pilot.md"),
        semantic_object_fixture(
            "task.pilot",
            "task",
            "Pilot Task",
            "active",
            "  - kind: constrains\n    target: invariant.runtime\n",
        ),
    );
    write_file(
        temp.path().join("objects/invariant/runtime.md"),
        semantic_object_fixture(
            "invariant.runtime",
            "invariant",
            "Runtime Invariant",
            "active",
            "",
        ),
    );
    write_file(
        temp.path().join("projections/llm-compression.md"),
        semantic_projection_fixture(
            &["task.pilot", "invariant.runtime"],
            "stale-fixture",
            "stale",
        ),
    );
    refresh_projection_as_fresh(temp.path(), &["task.pilot", "invariant.runtime"]);

    let repository = load_semantic_repository(temp.path());
    assert!(
        repository.report.is_success(),
        "repository should validate: {:?}",
        repository.report.issues
    );

    let bundle = semantic_scope_bundle(
        &repository,
        &SemanticScopeRequest {
            task_id: Some("task.pilot".to_string()),
            object_ids: Vec::new(),
        },
    );

    assert_eq!(
        bundle
            .objects
            .iter()
            .map(|object| object.id.as_str())
            .collect::<Vec<_>>(),
        vec!["invariant.runtime", "task.pilot"]
    );
    assert_eq!(bundle.affected_invariants, vec!["invariant.runtime"]);
    assert_eq!(bundle.projection_revision, "test.v1");
    assert!(bundle.projection_source_revision.is_some());
    let projection_staleness = serde_json::to_value(&bundle.projection_staleness)
        .unwrap_or_else(|error| panic!("serialize staleness: {error}"));
    assert_eq!(projection_staleness, serde_json::json!("fresh"));
}

#[test]
fn semantic_scope_metadata_envelope_round_trips_bundle_and_policy_evidence() {
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
    let bundle = semantic_scope_bundle(
        &repository,
        &SemanticScopeRequest {
            task_id: None,
            object_ids: vec!["component.test".to_string()],
        },
    );
    let policy_report = semantic_projection_freshness_policy_report(&repository);
    let envelope = semantic_scope_metadata_envelope(
        bundle.clone(),
        Some(serde_json::json!({
            "guardId": "semantic_sql.projection_freshness",
            "status": "review_required",
            "failingRowCount": 1,
            "message": "semantic projection freshness guard requires review"
        })),
        Some(policy_report),
    );

    let encoded = semantic_scope_metadata_envelope_to_vec(&envelope)
        .unwrap_or_else(|error| panic!("metadata envelope should encode: {error}"));
    let raw_metadata_json = std::str::from_utf8(&encoded)
        .unwrap_or_else(|error| panic!("metadata envelope should be UTF-8: {error}"));
    let decoded = parse_semantic_scope_metadata_envelope_json(raw_metadata_json)
        .unwrap_or_else(|error| panic!("metadata envelope should decode: {error}"));

    assert_eq!(decoded.bundle.projection_revision, "test.v1");
    assert_eq!(
        decoded
            .sql_guard_evidence
            .as_ref()
            .and_then(|value| value.get("status"))
            .and_then(serde_json::Value::as_str),
        Some("review_required")
    );
    let decoded_policy = decoded
        .projection_policy_evidence
        .as_ref()
        .unwrap_or_else(|| panic!("projection policy evidence should decode"));
    assert_eq!(
        decoded_policy.policy_id,
        SEMANTIC_PROJECTION_FRESHNESS_POLICY_ID
    );
    assert_eq!(decoded_policy.status, "review_required");

    let raw_bundle_json = serde_json::to_string(&bundle)
        .unwrap_or_else(|error| panic!("raw bundle should encode: {error}"));
    let decoded_raw_bundle = parse_semantic_scope_metadata_envelope_json(&raw_bundle_json)
        .unwrap_or_else(|error| panic!("raw bundle should decode: {error}"));
    assert_eq!(decoded_raw_bundle.bundle.projection_revision, "test.v1");
    assert!(decoded_raw_bundle.sql_guard_evidence.is_none());
    assert!(decoded_raw_bundle.projection_policy_evidence.is_none());
}

#[test]
fn semantic_scope_bundle_includes_related_change_intents() {
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
        "repository should validate: {:?}",
        repository.report.issues
    );

    let bundle = semantic_scope_bundle(
        &repository,
        &SemanticScopeRequest {
            task_id: None,
            object_ids: vec!["component.test".to_string()],
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
    assert!(
        bundle
            .required_validations
            .iter()
            .any(|validation| validation.contains("cargo test -p xiuxian-wendao-parsers semantic")),
        "change intent validations should be included: {:?}",
        bundle.required_validations
    );
}
