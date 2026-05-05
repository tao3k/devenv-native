use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use tempfile::tempdir;
use xiuxian_wendao_parsers::{
    SEMANTIC_PROJECTION_FRESHNESS_POLICY_ID, SemanticScopeRequest, load_semantic_repository,
    parse_semantic_object, semantic_projection_freshness_policy_report,
    semantic_projection_source_revision, semantic_scope_bundle,
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

fn semantic_object_fixture(
    id: &str,
    kind: &str,
    title: &str,
    status: &str,
    relations: &str,
) -> String {
    semantic_object_fixture_with_confidence(id, kind, title, status, "human_signed", relations)
}

fn semantic_object_fixture_with_confidence(
    id: &str,
    kind: &str,
    title: &str,
    status: &str,
    confidence_source: &str,
    relations: &str,
) -> String {
    format!(
        concat!(
            "---\n",
            "id: {id}\n",
            "kind: {kind}\n",
            "title: {title}\n",
            "status: {status}\n",
            "confidence:\n",
            "  score: 1.0\n",
            "  source: {confidence_source}\n",
            "owners:\n",
            "  - scope: packages/rust/crates/xiuxian-wendao-parsers\n",
            "    role: parser_owner\n",
            "provenance:\n",
            "  source: docs/rfcs/2026-05-03-repo-native-semantic-ssot-layer-rfc.md\n",
            "  recorded_by: codex\n",
            "  recorded_at: \"2026-05-05\"\n",
            "verification:\n",
            "  required:\n",
            "    - direnv exec . wendao-client lint semantic\n",
            "relations:\n",
            "{relations}",
            "---\n",
            "# {title}\n",
            "\n",
            "Fixture body.\n",
        ),
        id = id,
        kind = kind,
        title = title,
        status = status,
        confidence_source = confidence_source,
        relations = if relations.is_empty() {
            "  []\n"
        } else {
            relations
        },
    )
}

fn write_file(path: impl AsRef<Path>, content: impl AsRef<str>) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("fixture directory should be created: {error}"));
    }
    fs::write(path, content.as_ref())
        .unwrap_or_else(|error| panic!("fixture file should be written: {error}"));
}

fn semantic_projection_fixture(
    source_objects: &[&str],
    source_revision: &str,
    staleness: &str,
) -> String {
    let mut rendered_source_objects = String::new();
    for object_id in source_objects {
        writeln!(&mut rendered_source_objects, "  - {object_id}")
            .unwrap_or_else(|error| panic!("render projection source object: {error}"));
    }
    format!(
        concat!(
            "---\n",
            "type: semantic_projection\n",
            "projection: llm_compression\n",
            "source_objects:\n",
            "{rendered_source_objects}",
            "source_revision: {source_revision}\n",
            "projection_revision: test.v1\n",
            "staleness: {staleness}\n",
            "status: active\n",
            "---\n",
            "# Projection\n",
        ),
        rendered_source_objects = rendered_source_objects,
        source_revision = source_revision,
        staleness = staleness,
    )
}

fn refresh_projection_as_fresh(root: &Path, source_objects: &[&str]) {
    let stale_repository = load_semantic_repository(root);
    let Some(projection) = stale_repository.projections.first() else {
        panic!("projection fixture should load");
    };
    let source_revision = semantic_projection_source_revision(&stale_repository, projection)
        .unwrap_or_else(|| panic!("projection source revision should compute"));
    write_file(
        root.join("projections/llm-compression.md"),
        semantic_projection_fixture(source_objects, &source_revision, "fresh"),
    );
}

fn semantic_change_intent_fixture(
    touched_object: &str,
    affected_invariant: &str,
    relation_source: &str,
    relation_target: &str,
    projection: &str,
) -> String {
    semantic_change_intent_fixture_with_candidates(
        touched_object,
        affected_invariant,
        relation_source,
        relation_target,
        projection,
        &[],
    )
}

fn semantic_change_intent_fixture_with_candidates(
    touched_object: &str,
    affected_invariant: &str,
    relation_source: &str,
    relation_target: &str,
    projection: &str,
    candidate_suggestions: &[&str],
) -> String {
    semantic_change_intent_fixture_with_status_transitions(
        touched_object,
        affected_invariant,
        relation_source,
        relation_target,
        projection,
        candidate_suggestions,
        &[],
    )
}

fn semantic_change_intent_fixture_with_status_transitions(
    touched_object: &str,
    affected_invariant: &str,
    relation_source: &str,
    relation_target: &str,
    projection: &str,
    candidate_suggestions: &[&str],
    status_transitions: &[(&str, &str, &str)],
) -> String {
    semantic_change_intent_fixture_with_lifecycle_targets(SemanticChangeIntentFixture {
        touched_object,
        affected_invariant,
        relation_source,
        relation_target,
        projection,
        candidate_suggestions,
        status_transitions,
        promotion_targets: &[],
        demotion_targets: &[],
    })
}

#[derive(Clone, Copy)]
struct SemanticChangeIntentFixture<'a> {
    touched_object: &'a str,
    affected_invariant: &'a str,
    relation_source: &'a str,
    relation_target: &'a str,
    projection: &'a str,
    candidate_suggestions: &'a [&'a str],
    status_transitions: &'a [(&'a str, &'a str, &'a str)],
    promotion_targets: &'a [&'a str],
    demotion_targets: &'a [&'a str],
}

fn semantic_change_intent_fixture_with_lifecycle_targets(
    fixture: SemanticChangeIntentFixture<'_>,
) -> String {
    let mut rendered_candidate_suggestions = String::new();
    if fixture.candidate_suggestions.is_empty() {
        rendered_candidate_suggestions.push_str("[]\n");
    } else {
        rendered_candidate_suggestions.push('\n');
        for object_id in fixture.candidate_suggestions {
            writeln!(&mut rendered_candidate_suggestions, "  - {object_id}")
                .unwrap_or_else(|error| panic!("render candidate suggestion: {error}"));
        }
    }
    let mut rendered_status_transitions = String::new();
    if fixture.status_transitions.is_empty() {
        rendered_status_transitions.push_str("[]\n");
    } else {
        rendered_status_transitions.push('\n');
        for (object_id, from, to) in fixture.status_transitions {
            writeln!(
                &mut rendered_status_transitions,
                "  - object_id: {object_id}\n    from: {from}\n    to: {to}"
            )
            .unwrap_or_else(|error| panic!("render status transition: {error}"));
        }
    }
    let mut rendered_promotion_targets = String::new();
    if fixture.promotion_targets.is_empty() {
        rendered_promotion_targets.push_str("[]\n");
    } else {
        rendered_promotion_targets.push('\n');
        for object_id in fixture.promotion_targets {
            writeln!(&mut rendered_promotion_targets, "  - {object_id}")
                .unwrap_or_else(|error| panic!("render promotion target: {error}"));
        }
    }
    let mut rendered_demotion_targets = String::new();
    if fixture.demotion_targets.is_empty() {
        rendered_demotion_targets.push_str("[]\n");
    } else {
        rendered_demotion_targets.push('\n');
        for object_id in fixture.demotion_targets {
            writeln!(&mut rendered_demotion_targets, "  - {object_id}")
                .unwrap_or_else(|error| panic!("render demotion target: {error}"));
        }
    }
    format!(
        concat!(
            "---\n",
            "type: semantic_change_intent\n",
            "id: change.semantic-ssot.test\n",
            "title: Semantic SSOT Test Change\n",
            "status: active\n",
            "touched_objects:\n",
            "  - {touched_object}\n",
            "changed_relations:\n",
            "  - source: {relation_source}\n",
            "    kind: validates\n",
            "    target: {relation_target}\n",
            "    action: add\n",
            "status_transitions: {rendered_status_transitions}",
            "promotion_targets: {rendered_promotion_targets}",
            "demotion_targets: {rendered_demotion_targets}",
            "affected_invariants:\n",
            "  - {affected_invariant}\n",
            "required_validations:\n",
            "  - direnv exec . cargo test -p xiuxian-wendao-parsers semantic -- --nocapture\n",
            "projections_to_refresh:\n",
            "  - {projection}\n",
            "candidate_suggestions: {rendered_candidate_suggestions}",
            "---\n",
            "# Semantic SSOT Test Change\n",
        ),
        touched_object = fixture.touched_object,
        affected_invariant = fixture.affected_invariant,
        relation_source = fixture.relation_source,
        relation_target = fixture.relation_target,
        projection = fixture.projection,
        rendered_candidate_suggestions = rendered_candidate_suggestions,
        rendered_status_transitions = rendered_status_transitions,
        rendered_promotion_targets = rendered_promotion_targets,
        rendered_demotion_targets = rendered_demotion_targets,
    )
}
