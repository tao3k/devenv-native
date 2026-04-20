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
        .with_dmn_sources(vec![dmn_fixture_source("simple-unique-eligibility.dmn")]),
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

    assert_eq!(package.dmn_decisions().len(), 1);
    assert_eq!(decision.decision.decision_id.as_ref(), "loan-decision");
    assert_eq!(decision.source_id.as_ref(), "simple-unique-eligibility.dmn");
}

#[test]
fn parser_bundle_snapshot_surfaces_invalid_dmn_sources() {
    let error = parse_bpmn_bundle(
        &BpmnBundleSnapshot::new(vec![fixture_source(
            "linear-business-rule-placeholder.bpmn",
        )])
        .with_dmn_sources(vec![dmn_fixture_source("invalid-multiple-decisions.dmn")]),
        &BpmnParseOptions::default(),
    )
    .must_err("invalid bundled DMN should fail through typed DMN parse errors");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedDmnDecisionCount {
            source_id: "invalid-multiple-decisions.dmn".to_string(),
            count: 2,
        }
    );
}

fn dmn_fixture_source(name: &str) -> DmnSourceFile {
    let path = format!("{}/tests/fixtures/dmn/{name}", env!("CARGO_MANIFEST_DIR"));
    let contents =
        std::fs::read_to_string(path).must("fixture should be readable from the crate tree");
    DmnSourceFile::new(name, contents)
}
