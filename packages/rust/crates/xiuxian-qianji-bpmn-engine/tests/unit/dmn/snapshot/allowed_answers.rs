use crate::dmn::fixture_source;
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::snapshot_dmn_source;

#[test]
fn dmn_snapshot_classifies_allowed_answers_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "versioned-allowed-answers-decision-20191111.dmn",
    ))
    .must("allowed-answers DMN source should still produce a document snapshot");

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
        "Decision_allowed_answers"
    );
    assert_eq!(snapshot.decisions[0].allowed_answers_count, 1);
    assert_eq!(snapshot.decisions[0].decision_table_count, 0);
    assert_eq!(snapshot.decisions[0].literal_expression_count, 0);
    assert_eq!(snapshot.decisions[0].context_count, 0);
    assert_eq!(snapshot.decisions[0].invocation_count, 0);
    assert_eq!(snapshot.decisions[0].relation_count, 0);
    assert_eq!(snapshot.decisions[0].function_definition_count, 0);
    assert_eq!(snapshot.decisions[0].list_count, 0);
}
