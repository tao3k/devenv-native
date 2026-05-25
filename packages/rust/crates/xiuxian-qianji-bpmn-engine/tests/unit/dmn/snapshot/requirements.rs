use crate::dmn::fixture_source;
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::snapshot_dmn_source;

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
    assert_eq!(
        snapshot.root.business_knowledge_models[0]
            .body
            .as_ref()
            .and_then(|body| body.text.as_deref()),
        Some("\"external-policy\"")
    );
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
    assert_eq!(
        snapshot.decisions[0]
            .requirement_references
            .iter()
            .map(|reference| (
                reference.requirement_kind.as_str(),
                reference.reference_kind.as_str(),
                reference.href.as_deref(),
            ))
            .collect::<Vec<_>>(),
        vec![(
            "knowledgeRequirement",
            "requiredKnowledge",
            Some("#BKM_policy_source")
        )]
    );
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
    assert_eq!(
        snapshot.decisions[0]
            .requirement_references
            .iter()
            .map(|reference| (
                reference.requirement_kind.as_str(),
                reference.reference_kind.as_str(),
                reference.href.as_deref(),
            ))
            .collect::<Vec<_>>(),
        vec![(
            "authorityRequirement",
            "requiredAuthority",
            Some("#KnowledgeSource_policy")
        )]
    );
}

#[test]
fn dmn_snapshot_classifies_authority_requirement_decisions_with_mixed_targets() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "versioned-authority-requirement-mixed-references-20191111.dmn",
    ))
    .must("mixed authority-requirement DMN source should still produce a document snapshot");

    let decision = snapshot
        .decisions
        .iter()
        .find(|decision| decision.decision_id == "Decision_authority_requirement_mixed");
    let Some(decision) = decision else {
        panic!("mixed authority-requirement decision should be present");
    };

    assert_eq!(decision.information_requirement_count, 0);
    assert_eq!(decision.required_input_count, 1);
    assert_eq!(decision.required_decision_count, 1);
    assert_eq!(decision.knowledge_requirement_count, 0);
    assert_eq!(decision.required_knowledge_count, 0);
    assert_eq!(decision.authority_requirement_count, 1);
    assert_eq!(decision.required_authority_count, 1);
    assert_eq!(
        decision
            .requirement_references
            .iter()
            .map(|reference| (
                reference.requirement_kind.as_str(),
                reference.reference_kind.as_str(),
                reference.href.as_deref(),
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                "authorityRequirement",
                "requiredAuthority",
                Some("#KnowledgeSource_policy")
            ),
            (
                "authorityRequirement",
                "requiredDecision",
                Some("#Decision_upstream")
            ),
            (
                "authorityRequirement",
                "requiredInput",
                Some("#InputData_customer")
            ),
        ]
    );
}
