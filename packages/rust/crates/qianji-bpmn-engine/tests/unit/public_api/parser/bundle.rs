use super::super::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnBundleSnapshot, BpmnEngineError, BpmnParseOptions, DmnSourceFile, parse_bpmn_bundle,
};

#[test]
fn parser_bundle_snapshot_populates_package_dmn_registry() {
    let package = parse_bpmn_bundle(
        &BpmnBundleSnapshot::new(vec![fixture_source(
            "linear-business-rule-placeholder.bpmn",
        )])
        .with_dmn_sources(vec![dmn_fixture_source("multiple-decisions.dmn")]),
        &BpmnParseOptions::default(),
    )
    .must("bundle snapshot should populate the package DMN registry");

    let process = package
        .find_process("loan_review")
        .must("process should be present");
    let decision_ref = process.nodes[1]
        .decision
        .as_ref()
        .must("business rule task should keep the DMN reference");
    let decision = package
        .find_dmn_decision(decision_ref)
        .must("registered decision lookup should succeed")
        .must("registered decision should be present");

    let secondary = package
        .find_dmn_decision(
            &qianji_bpmn_engine::DmnDecisionRef::new("secondary-review")
                .with_source_id("multiple-decisions.dmn"),
        )
        .must("secondary decision lookup should succeed")
        .must("secondary decision should be present");

    assert_eq!(package.dmn_decisions().len(), 2);
    assert_eq!(decision.decision.decision_id.as_ref(), "loan-decision");
    assert_eq!(decision.source_id.as_ref(), "multiple-decisions.dmn");
    assert_eq!(secondary.decision.decision_id.as_ref(), "secondary-review");
}

#[test]
fn parser_bundle_snapshot_populates_package_dmn_input_data_registry() {
    let package = parse_bpmn_bundle(
        &BpmnBundleSnapshot::new(vec![fixture_source(
            "linear-business-rule-placeholder.bpmn",
        )])
        .with_dmn_sources(vec![dmn_fixture_source(
            "versioned-local-required-input-runtime-20191111.dmn",
        )]),
        &BpmnParseOptions::default(),
    )
    .must("bundle snapshot should populate the package DMN input-data registry");

    let input_data = package
        .find_dmn_input_data(
            "versioned-local-required-input-runtime-20191111.dmn",
            "InputData_applicant",
        )
        .must("registered input-data should be present");

    assert_eq!(package.dmn_decisions().len(), 1);
    assert_eq!(package.dmn_input_data().len(), 1);
    assert_eq!(input_data.name.as_deref(), Some("applicant_input"));
    assert_eq!(input_data.variable_name.as_deref(), Some("applicant"));
}

#[test]
fn parser_bundle_snapshot_populates_package_dmn_business_knowledge_model_registry() {
    let package = parse_bpmn_bundle(
        &BpmnBundleSnapshot::new(vec![fixture_source(
            "linear-business-rule-placeholder.bpmn",
        )])
        .with_dmn_sources(vec![dmn_fixture_source(
            "versioned-business-knowledge-model-registry-source-20191111.dmn",
        )]),
        &BpmnParseOptions::default(),
    )
    .must("bundle snapshot should populate the package DMN BKM registry");

    let business_knowledge_model = package
        .find_dmn_business_knowledge_model(
            "versioned-business-knowledge-model-registry-source-20191111.dmn",
            "BKM_policy_source",
        )
        .must("registered BKM should be present");

    assert_eq!(package.dmn_decisions().len(), 1);
    assert_eq!(package.dmn_business_knowledge_models().len(), 1);
    assert_eq!(
        business_knowledge_model.name.as_deref(),
        Some("Policy Source")
    );
    assert_eq!(
        business_knowledge_model.variable_name.as_deref(),
        Some("policy")
    );
    assert_eq!(
        business_knowledge_model
            .encapsulated_logic
            .as_ref()
            .and_then(|logic| logic.kind.as_deref()),
        Some("FEEL")
    );
}

#[test]
fn parser_bundle_snapshot_surfaces_invalid_dmn_sources() {
    let error = parse_bpmn_bundle(
        &BpmnBundleSnapshot::new(vec![fixture_source(
            "linear-business-rule-placeholder.bpmn",
        )])
        .with_dmn_sources(vec![dmn_fixture_source(
            "invalid-unsupported-unary-test.dmn",
        )]),
        &BpmnParseOptions::default(),
    )
    .must_err("invalid bundled DMN should fail through typed DMN parse errors");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedDmnUnaryTest {
            source_id: "invalid-unsupported-unary-test.dmn".to_string(),
            expression: "duration(\"P1.5Y\")".to_string(),
        }
    );
}

fn dmn_fixture_source(name: &str) -> DmnSourceFile {
    let path = format!("{}/tests/fixtures/dmn/{name}", env!("CARGO_MANIFEST_DIR"));
    let contents =
        std::fs::read_to_string(path).must("fixture should be readable from the crate tree");
    DmnSourceFile::new(name, contents)
}
