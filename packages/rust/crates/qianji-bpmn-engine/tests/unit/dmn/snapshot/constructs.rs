use super::super::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::snapshot_dmn_source;

#[test]
fn dmn_snapshot_classifies_literal_expression_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "versioned-literal-expression-decision-20191111.dmn",
    ))
    .must("literal-expression DMN source should still produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20191111")
    );
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(
        snapshot.decisions[0].decision_id,
        "Decision_literal_expression"
    );
    assert_eq!(snapshot.decisions[0].decision_table_count, 0);
    assert_eq!(snapshot.decisions[0].literal_expression_count, 1);
    assert_eq!(snapshot.decisions[0].context_count, 0);
    assert_eq!(snapshot.decisions[0].invocation_count, 0);
    assert_eq!(snapshot.decisions[0].relation_count, 0);
    assert_eq!(snapshot.decisions[0].function_definition_count, 0);
    assert_eq!(snapshot.decisions[0].list_count, 0);
}

#[test]
fn dmn_snapshot_classifies_context_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source("versioned-context-decision-20191111.dmn"))
        .must("context DMN source should still produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20191111")
    );
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(snapshot.decisions[0].decision_id, "Decision_context");
    assert_eq!(snapshot.decisions[0].decision_table_count, 0);
    assert_eq!(snapshot.decisions[0].literal_expression_count, 0);
    assert_eq!(snapshot.decisions[0].context_count, 1);
    assert_eq!(snapshot.decisions[0].invocation_count, 0);
    assert_eq!(snapshot.decisions[0].relation_count, 0);
    assert_eq!(snapshot.decisions[0].function_definition_count, 0);
    assert_eq!(snapshot.decisions[0].list_count, 0);
}

#[test]
fn dmn_snapshot_classifies_invocation_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "versioned-invocation-decision-20191111.dmn",
    ))
    .must("invocation DMN source should still produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20191111")
    );
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(snapshot.decisions[0].decision_id, "Decision_invocation");
    assert_eq!(snapshot.decisions[0].decision_table_count, 0);
    assert_eq!(snapshot.decisions[0].literal_expression_count, 0);
    assert_eq!(snapshot.decisions[0].context_count, 0);
    assert_eq!(snapshot.decisions[0].invocation_count, 1);
    assert_eq!(snapshot.decisions[0].relation_count, 0);
    assert_eq!(snapshot.decisions[0].function_definition_count, 0);
    assert_eq!(snapshot.decisions[0].list_count, 0);
}

#[test]
fn dmn_snapshot_classifies_relation_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source("versioned-relation-decision-20191111.dmn"))
        .must("relation DMN source should still produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20191111")
    );
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(snapshot.decisions[0].decision_id, "Decision_relation");
    assert_eq!(snapshot.decisions[0].decision_table_count, 0);
    assert_eq!(snapshot.decisions[0].literal_expression_count, 0);
    assert_eq!(snapshot.decisions[0].context_count, 0);
    assert_eq!(snapshot.decisions[0].invocation_count, 0);
    assert_eq!(snapshot.decisions[0].relation_count, 1);
    assert_eq!(snapshot.decisions[0].function_definition_count, 0);
    assert_eq!(snapshot.decisions[0].list_count, 0);
}

#[test]
fn dmn_snapshot_classifies_function_definition_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "versioned-function-definition-decision-20191111.dmn",
    ))
    .must("function-definition DMN source should still produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20191111")
    );
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(
        snapshot.decisions[0].decision_id,
        "Decision_function_definition"
    );
    assert_eq!(snapshot.decisions[0].decision_table_count, 0);
    assert_eq!(snapshot.decisions[0].literal_expression_count, 0);
    assert_eq!(snapshot.decisions[0].context_count, 0);
    assert_eq!(snapshot.decisions[0].invocation_count, 0);
    assert_eq!(snapshot.decisions[0].relation_count, 0);
    assert_eq!(snapshot.decisions[0].function_definition_count, 1);
    assert_eq!(snapshot.decisions[0].list_count, 0);
}

#[test]
fn dmn_snapshot_classifies_list_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source("versioned-list-decision-20191111.dmn"))
        .must("list DMN source should still produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20191111")
    );
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(snapshot.decisions[0].decision_id, "Decision_list");
    assert_eq!(snapshot.decisions[0].decision_table_count, 0);
    assert_eq!(snapshot.decisions[0].literal_expression_count, 0);
    assert_eq!(snapshot.decisions[0].context_count, 0);
    assert_eq!(snapshot.decisions[0].invocation_count, 0);
    assert_eq!(snapshot.decisions[0].relation_count, 0);
    assert_eq!(snapshot.decisions[0].function_definition_count, 0);
    assert_eq!(snapshot.decisions[0].list_count, 1);
}
