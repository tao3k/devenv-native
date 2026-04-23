use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnEngineError, BpmnPackage, DmnBusinessKnowledgeModelDefinition, DmnDecisionRef,
    DmnInformationRequirementReference, DmnSourceFile, parse_dmn_decision, parse_dmn_decisions,
    snapshot_dmn_source,
};

#[test]
fn package_registered_dmn_decision_resolves_deterministically() {
    let definition = parse_dmn_decision(&fixture_source(
        "simple-unique-eligibility.dmn",
        "simple-unique-eligibility.dmn",
    ))
    .must("fixture DMN should parse");
    let package = BpmnPackage::new("pkg_api", Vec::new()).with_dmn_decisions(vec![definition]);

    let resolved = package
        .find_dmn_decision(
            &DmnDecisionRef::new("loan-decision").with_source_id("simple-unique-eligibility.dmn"),
        )
        .must("registered decision lookup should stay deterministic")
        .must("registered decision should resolve");

    assert_eq!(resolved.decision.decision_id.as_ref(), "loan-decision");
    assert_eq!(package.dmn_decisions().len(), 1);
}

#[test]
fn package_unqualified_duplicate_dmn_decision_ref_is_rejected() {
    let first = parse_dmn_decision(&fixture_source(
        "first.dmn",
        "simple-unique-eligibility.dmn",
    ))
    .must("first DMN source should parse");
    let second = parse_dmn_decision(&fixture_source(
        "second.dmn",
        "simple-unique-eligibility.dmn",
    ))
    .must("second DMN source should parse");
    let package = BpmnPackage::new("pkg_api", Vec::new()).with_dmn_decisions(vec![first, second]);

    let error = package
        .find_dmn_decision(&DmnDecisionRef::new("loan-decision"))
        .must_err("unqualified duplicate decision ids should fail explicitly");

    assert_eq!(
        error,
        BpmnEngineError::AmbiguousDmnDecisionReference {
            decision_id: "loan-decision".to_string(),
            source_id: None,
            count: 2,
            source_suffix: String::new(),
        }
    );
}

#[test]
fn package_same_source_multi_decision_lookup_resolves_deterministically() {
    let definitions = parse_dmn_decisions(&fixture_source(
        "multiple-decisions.dmn",
        "multiple-decisions.dmn",
    ))
    .must("multi-decision source should parse through the plural API");
    let package = BpmnPackage::new("pkg_api", Vec::new()).with_dmn_decisions(definitions);

    let resolved = package
        .find_dmn_decision(
            &DmnDecisionRef::new("secondary-review").with_source_id("multiple-decisions.dmn"),
        )
        .must("lookup should succeed")
        .must("secondary decision should resolve");

    assert_eq!(package.dmn_decisions().len(), 2);
    assert_eq!(resolved.decision.decision_id.as_ref(), "secondary-review");
    assert_eq!(resolved.source_id.as_ref(), "multiple-decisions.dmn");
}

#[test]
fn package_registered_dmn_decision_preserves_information_requirement_contract() {
    let definitions = parse_dmn_decisions(&fixture_source(
        "executable-information-requirements.dmn",
        "versioned-executable-information-requirements-20191111.dmn",
    ))
    .must("executable information-requirement source should parse");
    let package = BpmnPackage::new("pkg_api", Vec::new()).with_dmn_decisions(definitions);

    let resolved = package
        .find_dmn_decision(
            &DmnDecisionRef::new("Decision_executable_dependency")
                .with_source_id("executable-information-requirements.dmn"),
        )
        .must("registered decision lookup should succeed")
        .must("registered decision should resolve");

    assert_eq!(
        resolved.information_requirements,
        vec![
            DmnInformationRequirementReference::new("requiredInput", Some("#InputData_customer")),
            DmnInformationRequirementReference::new("requiredDecision", Some("#Decision_upstream")),
        ]
    );
}

#[test]
fn package_registered_dmn_business_knowledge_model_resolves_deterministically() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "metadata-only-business-knowledge-model-invocable-20191111.dmn",
        "metadata-only-business-knowledge-model-invocable-20191111.dmn",
    ))
    .must("BKM fixture should snapshot");
    let definition = DmnBusinessKnowledgeModelDefinition::from_snapshot(
        "metadata-only-business-knowledge-model-invocable-20191111.dmn",
        snapshot
            .root
            .business_knowledge_models
            .first()
            .must("fixture should contain one top-level BKM"),
    );
    let package = BpmnPackage::new("pkg_api", Vec::new())
        .with_dmn_business_knowledge_models(vec![definition]);

    let resolved = package
        .find_dmn_business_knowledge_model(
            "metadata-only-business-knowledge-model-invocable-20191111.dmn",
            "BKM_policy_source",
        )
        .must("registered BKM should resolve");

    assert_eq!(package.dmn_business_knowledge_models().len(), 1);
    assert_eq!(resolved.name.as_deref(), Some("Policy Source"));
    assert_eq!(resolved.variable_name.as_deref(), Some("policy"));
    assert_eq!(resolved.variable_type_ref.as_deref(), Some("string"));
    assert_eq!(
        resolved
            .encapsulated_logic
            .as_ref()
            .map(|logic| logic.parameters.len()),
        Some(1)
    );
}

fn fixture_source(source_id: &str, fixture_name: &str) -> DmnSourceFile {
    let path = format!(
        "{}/tests/fixtures/dmn/{fixture_name}",
        env!("CARGO_MANIFEST_DIR")
    );
    let contents = std::fs::read_to_string(path).must("fixture should be readable");
    DmnSourceFile::new(source_id, contents)
}
