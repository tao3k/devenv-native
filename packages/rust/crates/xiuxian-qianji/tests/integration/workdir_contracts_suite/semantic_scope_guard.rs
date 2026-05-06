use xiuxian_qianji::{
    WorkdirSemanticScopeGuardStatus, render_workdir_semantic_scope_guard_trace,
    trace_workdir_semantic_scope_json,
};

use super::{
    semantic_scope_metadata_json, semantic_scope_metadata_with_projection_policy_json,
    semantic_scope_metadata_with_sql_guard_json,
};

#[test]
fn workdir_semantic_scope_guard_trace_consumes_wendao_metadata_bundle() {
    let trace = trace_workdir_semantic_scope_json(&semantic_scope_metadata_json("fresh", &[]))
        .unwrap_or_else(|error| panic!("semantic-scope metadata should decode: {error}"));

    assert_eq!(trace.status, WorkdirSemanticScopeGuardStatus::Ready);
    assert_eq!(trace.task_id.as_deref(), Some("task.demo"));
    assert_eq!(trace.relation_count, 1);
    assert_eq!(trace.change_intent_ids, vec!["change.demo"]);
    assert!(
        trace
            .objects
            .iter()
            .any(|object| object.id == "task.demo" && object.status == "candidate")
    );
    assert!(
        trace
            .required_validations
            .contains(&"cargo test -p xiuxian-qianji workdir_semantic".to_string())
    );

    let rendered = render_workdir_semantic_scope_guard_trace(&trace);
    assert!(rendered.contains("Status: ready"));
    assert!(rendered.contains("task.demo [task / candidate]"));
    assert!(rendered.contains("change.demo"));
}
#[test]
fn workdir_semantic_scope_guard_trace_marks_stale_projection_for_review() {
    let trace = trace_workdir_semantic_scope_json(&semantic_scope_metadata_json("stale", &[]))
        .unwrap_or_else(|error| panic!("stale semantic-scope metadata should decode: {error}"));

    assert_eq!(
        trace.status,
        WorkdirSemanticScopeGuardStatus::ReviewRequired
    );
    assert!(
        trace
            .issues
            .iter()
            .any(|issue| issue.contains("semantic projection is stale"))
    );
}
#[test]
fn workdir_semantic_scope_guard_trace_consumes_sql_guard_review_evidence() {
    let trace = trace_workdir_semantic_scope_json(&semantic_scope_metadata_with_sql_guard_json(
        "review_required",
        1,
        "semantic projection freshness guard requires review: 1 stale projection row(s)",
    ))
    .unwrap_or_else(|error| panic!("semantic SQL guard evidence should decode: {error}"));

    assert_eq!(
        trace.status,
        WorkdirSemanticScopeGuardStatus::ReviewRequired
    );
    assert_eq!(trace.sql_guard_evidence.len(), 1);
    assert_eq!(
        trace.sql_guard_evidence[0].guard_id,
        "semantic_sql.projection_freshness"
    );
    assert_eq!(trace.sql_guard_evidence[0].failing_row_count, 1);
    assert!(
        trace
            .issues
            .iter()
            .any(|issue| issue.contains("semantic_sql.projection_freshness"))
    );

    let rendered = render_workdir_semantic_scope_guard_trace(&trace);
    assert!(rendered.contains("## SQL Guard Evidence"));
    assert!(rendered.contains("semantic_sql.projection_freshness"));
}
#[test]
fn workdir_semantic_scope_guard_trace_keeps_passed_sql_guard_ready() {
    let trace = trace_workdir_semantic_scope_json(&semantic_scope_metadata_with_sql_guard_json(
        "passed",
        0,
        "semantic projection freshness guard passed: no stale projection rows",
    ))
    .unwrap_or_else(|error| panic!("passed semantic SQL guard evidence should decode: {error}"));

    assert_eq!(trace.status, WorkdirSemanticScopeGuardStatus::Ready);
    assert_eq!(trace.sql_guard_evidence.len(), 1);
    assert!(trace.issues.is_empty());
}
#[test]
fn workdir_semantic_scope_guard_trace_consumes_projection_policy_review_evidence() {
    let trace = trace_workdir_semantic_scope_json(
        &semantic_scope_metadata_with_projection_policy_json(
            "review_required",
            1,
            "active change-intent projection refresh target(s) are stale",
        ),
    )
    .unwrap_or_else(|error| panic!("semantic projection policy evidence should decode: {error}"));

    assert_eq!(
        trace.status,
        WorkdirSemanticScopeGuardStatus::ReviewRequired
    );
    assert_eq!(trace.projection_policy_evidence.len(), 1);
    assert_eq!(
        trace.projection_policy_evidence[0].policy_id,
        "semantic_projection.required_refresh_targets"
    );
    assert_eq!(
        trace.projection_policy_evidence[0].failing_projection_count,
        1
    );
    assert!(
        trace
            .issues
            .iter()
            .any(|issue| issue.contains("semantic_projection.required_refresh_targets"))
    );

    let rendered = render_workdir_semantic_scope_guard_trace(&trace);
    assert!(rendered.contains("## Projection Policy Evidence"));
    assert!(rendered.contains("semantic_projection.required_refresh_targets"));
}
#[test]
fn workdir_semantic_scope_guard_trace_keeps_passed_projection_policy_ready() {
    let trace =
        trace_workdir_semantic_scope_json(&semantic_scope_metadata_with_projection_policy_json(
            "passed",
            0,
            "all active change-intent projection refresh targets are fresh",
        ))
        .unwrap_or_else(|error| panic!("passed projection policy evidence should decode: {error}"));

    assert_eq!(trace.status, WorkdirSemanticScopeGuardStatus::Ready);
    assert_eq!(trace.projection_policy_evidence.len(), 1);
    assert!(trace.issues.is_empty());
}
#[test]
fn workdir_semantic_scope_guard_trace_blocks_unresolved_ids() {
    let trace = trace_workdir_semantic_scope_json(&semantic_scope_metadata_json(
        "fresh",
        &["decision.missing"],
    ))
    .unwrap_or_else(|error| panic!("unresolved semantic-scope metadata should decode: {error}"));

    assert_eq!(trace.status, WorkdirSemanticScopeGuardStatus::Blocked);
    assert_eq!(trace.unresolved_ids, vec!["decision.missing"]);
    assert!(
        trace
            .issues
            .iter()
            .any(|issue| issue.contains("decision.missing"))
    );
}
