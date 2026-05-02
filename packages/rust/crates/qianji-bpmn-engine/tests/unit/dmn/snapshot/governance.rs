use crate::dmn::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::snapshot_dmn_source;

#[test]
fn dmn_snapshot_classifies_decision_maker_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "versioned-decision-maker-decision-20191111.dmn",
    ))
    .must("decision-maker DMN source should still produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20191111")
    );
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(snapshot.decisions[0].decision_id, "Decision_decision_maker");
    assert_eq!(snapshot.decisions[0].allowed_answers_count, 0);
    assert_eq!(snapshot.decisions[0].decision_maker_count, 1);
    assert_eq!(snapshot.decisions[0].decision_owner_count, 0);
    assert_eq!(snapshot.decisions[0].decision_table_count, 0);
}

#[test]
fn dmn_snapshot_classifies_decision_owner_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "versioned-decision-owner-decision-20191111.dmn",
    ))
    .must("decision-owner DMN source should still produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20191111")
    );
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(snapshot.decisions[0].decision_id, "Decision_decision_owner");
    assert_eq!(snapshot.decisions[0].allowed_answers_count, 0);
    assert_eq!(snapshot.decisions[0].decision_maker_count, 0);
    assert_eq!(snapshot.decisions[0].decision_owner_count, 1);
    assert_eq!(snapshot.decisions[0].decision_table_count, 0);
}

#[test]
fn dmn_snapshot_classifies_mixed_decision_governance_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "versioned-mixed-decision-governance-decision-20191111.dmn",
    ))
    .must("mixed-governance DMN source should still produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20191111")
    );
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(
        snapshot.decisions[0].decision_id,
        "Decision_mixed_decision_governance"
    );
    assert_eq!(snapshot.decisions[0].allowed_answers_count, 0);
    assert_eq!(snapshot.decisions[0].decision_maker_count, 1);
    assert_eq!(snapshot.decisions[0].decision_owner_count, 1);
    assert_eq!(snapshot.decisions[0].decision_table_count, 0);
}
