use crate::public_api::fixture_source;
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
fn parser_bundle_snapshot_populates_package_dmn_source_root_registry() {
    let package = parse_bpmn_bundle(
        &BpmnBundleSnapshot::new(vec![fixture_source(
            "linear-business-rule-placeholder.bpmn",
        )])
        .with_dmn_sources(vec![dmn_fixture_source(
            "versioned-local-decision-service-runtime-20191111.dmn",
        )]),
        &BpmnParseOptions::default(),
    )
    .must("bundle snapshot should populate the package DMN source-root registry");

    let source = package
        .find_dmn_source_definition_by_namespace(
            "https://example.com/dmn/local-decision-service-runtime",
        )
        .must("source namespace lookup should be deterministic")
        .must("registered source root should resolve");

    assert_eq!(package.dmn_source_definitions().len(), 1);
    assert_eq!(
        source.source_id.as_ref(),
        "versioned-local-decision-service-runtime-20191111.dmn"
    );
    assert_eq!(
        source.definitions_id.as_deref(),
        Some("Definitions_local_decision_service_runtime")
    );
    assert_eq!(
        source.name.as_deref(),
        Some("Local Decision Service Runtime")
    );
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
fn parser_bundle_snapshot_keeps_local_invocation_and_bkm_contract_together() {
    let package = parse_bpmn_bundle(
        &BpmnBundleSnapshot::new(vec![fixture_source(
            "linear-business-rule-placeholder.bpmn",
        )])
        .with_dmn_sources(vec![dmn_fixture_source(
            "versioned-local-bkm-invocation-runtime-20191111.dmn",
        )]),
        &BpmnParseOptions::default(),
    )
    .must("bundle snapshot should keep local invocation and BKM contracts together");

    let decision = package
        .find_dmn_decision(
            &qianji_bpmn_engine::DmnDecisionRef::new("Decision_invocation_runtime")
                .with_source_id("versioned-local-bkm-invocation-runtime-20191111.dmn"),
        )
        .must("registered invocation decision lookup should succeed")
        .must("registered invocation decision should be present");

    assert_eq!(package.dmn_decisions().len(), 1);
    assert_eq!(package.dmn_business_knowledge_models().len(), 1);
    assert_eq!(
        decision
            .invocation
            .as_ref()
            .and_then(|invocation| invocation.invoked_expression.as_ref())
            .map(|expression| expression.text.as_ref()),
        Some("scoreCard")
    );
}

#[test]
fn parser_bundle_snapshot_populates_package_dmn_decision_service_registry() {
    let package = parse_bpmn_bundle(
        &BpmnBundleSnapshot::new(vec![fixture_source(
            "linear-business-rule-placeholder.bpmn",
        )])
        .with_dmn_sources(vec![dmn_fixture_source(
            "versioned-local-decision-service-runtime-20191111.dmn",
        )]),
        &BpmnParseOptions::default(),
    )
    .must("bundle snapshot should populate the package decision-service registry");

    let decision_service = package
        .find_dmn_decision_service(
            &qianji_bpmn_engine::DmnDecisionRef::new("DecisionService_credit")
                .with_source_id("versioned-local-decision-service-runtime-20191111.dmn"),
        )
        .must("registered decision-service lookup should succeed")
        .must("registered decision service should be present");

    assert_eq!(package.dmn_decision_services().len(), 1);
    assert_eq!(
        decision_service.name.as_deref(),
        Some("Credit Decision Service")
    );
    assert_eq!(decision_service.output_decisions.len(), 1);
    assert_eq!(
        decision_service.output_decisions[0].href.as_deref(),
        Some("#Decision_approval")
    );
}

#[test]
fn parser_bundle_snapshot_keeps_imported_dmn_source_metadata_only() {
    let package = parse_bpmn_bundle(
        &BpmnBundleSnapshot::new(vec![fixture_source(
            "linear-business-rule-placeholder.bpmn",
        )])
        .with_dmn_sources(vec![dmn_fixture_source(
            "unsupported-top-level-import-20191111.dmn",
        )]),
        &BpmnParseOptions::default(),
    )
    .must("imported DMN source should load as metadata-only package state");

    assert!(package.dmn_decisions().is_empty());
    assert!(package.dmn_input_data().is_empty());
    assert!(package.dmn_business_knowledge_models().is_empty());
    assert!(package.dmn_decision_services().is_empty());
    assert_eq!(package.dmn_source_definitions().len(), 1);
    assert_eq!(package.dmn_imports().len(), 1);

    let source = package
        .find_dmn_source_definition_by_namespace(
            "https://example.com/dmn/unsupported-top-level-import",
        )
        .must("source namespace lookup should stay deterministic")
        .must("imported source root metadata should be registered");
    assert_eq!(
        source.source_id.as_ref(),
        "unsupported-top-level-import-20191111.dmn"
    );

    let bindings = package
        .dmn_import_source_bindings()
        .must("import source binding report should remain deterministic");
    assert_eq!(bindings.len(), 1);
    assert!(!bindings[0].is_bound());
    assert_eq!(
        bindings[0].dmn_import.namespace.as_deref(),
        Some("https://example.com/dmn/partner-services")
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
