use super::super::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::snapshot_dmn_source;

#[test]
fn dmn_snapshot_classifies_knowledge_requirement_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "versioned-knowledge-requirement-decision-20191111.dmn",
    ))
    .must("knowledge-requirement DMN source should still produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20191111")
    );
    assert_eq!(snapshot.root.import_count, 0);
    assert_eq!(snapshot.root.item_definition_count, 0);
    assert_eq!(snapshot.root.input_data_count, 0);
    assert_eq!(snapshot.root.knowledge_source_count, 0);
    assert_eq!(snapshot.root.business_knowledge_model_count, 1);
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.root.organization_unit_count, 0);
    assert_eq!(snapshot.root.performance_indicator_count, 0);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(
        snapshot.decisions[0].decision_id,
        "Decision_knowledge_requirement"
    );
    assert_eq!(snapshot.decisions[0].decision_table_count, 0);
    assert_eq!(snapshot.decisions[0].information_requirement_count, 0);
    assert_eq!(snapshot.decisions[0].required_input_count, 0);
    assert_eq!(snapshot.decisions[0].required_decision_count, 0);
    assert_eq!(snapshot.decisions[0].knowledge_requirement_count, 1);
    assert_eq!(snapshot.decisions[0].required_knowledge_count, 1);
    assert_eq!(snapshot.decisions[0].authority_requirement_count, 0);
    assert_eq!(snapshot.decisions[0].required_authority_count, 0);
}

#[test]
fn dmn_snapshot_classifies_authority_requirement_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "versioned-authority-requirement-decision-20191111.dmn",
    ))
    .must("authority-requirement DMN source should still produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20191111")
    );
    assert_eq!(snapshot.root.import_count, 0);
    assert_eq!(snapshot.root.item_definition_count, 0);
    assert_eq!(snapshot.root.input_data_count, 0);
    assert_eq!(snapshot.root.knowledge_source_count, 1);
    assert_eq!(snapshot.root.business_knowledge_model_count, 0);
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.root.organization_unit_count, 0);
    assert_eq!(snapshot.root.performance_indicator_count, 0);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(
        snapshot.decisions[0].decision_id,
        "Decision_authority_requirement"
    );
    assert_eq!(snapshot.decisions[0].decision_table_count, 0);
    assert_eq!(snapshot.decisions[0].information_requirement_count, 0);
    assert_eq!(snapshot.decisions[0].required_input_count, 0);
    assert_eq!(snapshot.decisions[0].required_decision_count, 0);
    assert_eq!(snapshot.decisions[0].knowledge_requirement_count, 0);
    assert_eq!(snapshot.decisions[0].required_knowledge_count, 0);
    assert_eq!(snapshot.decisions[0].authority_requirement_count, 1);
    assert_eq!(snapshot.decisions[0].required_authority_count, 1);
}
