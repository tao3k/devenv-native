use crate::dmn::{assert_dmn_json_snapshot, fixture_source};
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::{
    DmnHitPolicy, DmnInformationRequirementReference, DmnKnowledgeRequirementReference,
    parse_dmn_decision, parse_dmn_decisions,
};

#[test]
fn dmn_parser_single_decision_table_materializes_contract() {
    let decision = parse_dmn_decision(&fixture_source("simple-unique-eligibility.dmn"))
        .must("bounded DMN source should parse");

    assert_eq!(decision.table.hit_policy, DmnHitPolicy::Unique);
    assert_eq!(decision.table.inputs[0].lookup_path(), Some("tier"));
    assert_eq!(decision.table.inputs[0].type_ref.as_deref(), Some("string"));
    assert_eq!(
        decision.table.outputs[0].type_ref.as_deref(),
        Some("string")
    );
    assert_dmn_json_snapshot("simple_unique_eligibility_contract", &decision);
}

#[test]
fn dmn_parser_multiple_decisions_materialize_plural_contract() {
    let decisions = parse_dmn_decisions(&fixture_source("multiple-decisions.dmn"))
        .must("multi-decision DMN source should parse through the plural API");

    assert_eq!(decisions.len(), 2);
    assert_eq!(decisions[0].decision.decision_id.as_ref(), "loan-decision");
    assert_eq!(
        decisions[1].decision.decision_id.as_ref(),
        "secondary-review"
    );
    assert_eq!(decisions[0].source_id.as_ref(), "multiple-decisions.dmn");
}

#[test]
fn dmn_parser_preserves_executable_information_requirement_contract() {
    let decisions = parse_dmn_decisions(&fixture_source(
        "versioned-executable-information-requirements-20191111.dmn",
    ))
    .must("executable information-requirement source should parse through the plural API");

    let decision = decisions.iter().find(|decision| {
        decision.decision.decision_id.as_ref() == "Decision_executable_dependency"
    });
    let Some(decision) = decision else {
        panic!("executable dependency decision should be present");
    };

    assert_eq!(
        decision.information_requirements,
        vec![
            DmnInformationRequirementReference::new("requiredInput", Some("#InputData_customer")),
            DmnInformationRequirementReference::new("requiredDecision", Some("#Decision_upstream")),
        ]
    );
}

#[test]
fn dmn_parser_preserves_executable_knowledge_requirement_contract() {
    let decision = parse_dmn_decision(&fixture_source(
        "versioned-local-required-knowledge-runtime-20191111.dmn",
    ))
    .must("required-knowledge invocation source should parse");

    assert_eq!(
        decision.knowledge_requirements,
        vec![DmnKnowledgeRequirementReference::new(
            "requiredKnowledge",
            Some("#BKM_score_card"),
        )]
    );
}
