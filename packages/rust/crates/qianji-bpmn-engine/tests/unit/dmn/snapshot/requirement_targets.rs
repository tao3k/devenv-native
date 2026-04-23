use super::super::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::snapshot_dmn_source;

#[test]
fn dmn_snapshot_classifies_required_input_targets() {
    let snapshot = snapshot_dmn_source(&fixture_source("versioned-listed-input-data-20191111.dmn"))
        .must("required-input DMN source should still produce a document snapshot");

    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(snapshot.decisions[0].decision_id, "Decision_1");
    assert_eq!(snapshot.decisions[0].information_requirement_count, 1);
    assert_eq!(snapshot.decisions[0].required_input_count, 1);
    assert_eq!(snapshot.decisions[0].required_decision_count, 0);
    assert_eq!(snapshot.decisions[0].required_knowledge_count, 0);
    assert_eq!(snapshot.decisions[0].required_authority_count, 0);
}

#[test]
fn dmn_snapshot_classifies_required_decision_targets() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "versioned-required-decision-dependency-20191111.dmn",
    ))
    .must("required-decision DMN source should still produce a document snapshot");

    assert_eq!(snapshot.decisions.len(), 2);
    assert_eq!(
        snapshot.decisions[1].decision_id,
        "Decision_required_decision_dependency"
    );
    assert_eq!(snapshot.decisions[1].information_requirement_count, 1);
    assert_eq!(snapshot.decisions[1].required_input_count, 0);
    assert_eq!(snapshot.decisions[1].required_decision_count, 1);
    assert_eq!(snapshot.decisions[1].required_knowledge_count, 0);
    assert_eq!(snapshot.decisions[1].required_authority_count, 0);
}
