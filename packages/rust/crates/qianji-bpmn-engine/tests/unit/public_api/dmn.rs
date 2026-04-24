use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnEngineError, BpmnPackage, DmnBusinessKnowledgeModelDefinition, DmnDecisionRef,
    DmnDecisionServiceDefinition, DmnDecisionServiceReference, DmnImportDefinition,
    DmnInformationRequirementReference, DmnKnowledgeRequirementReference, DmnSourceDefinition,
    DmnSourceFile, parse_dmn_decision, parse_dmn_decisions, snapshot_dmn_source,
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
fn package_registered_dmn_decision_preserves_invocation_contract() {
    let definition = parse_dmn_decision(&fixture_source(
        "versioned-invocation-decision-20191111.dmn",
        "versioned-invocation-decision-20191111.dmn",
    ))
    .must("invocation DMN source should parse");
    let package = BpmnPackage::new("pkg_api", Vec::new()).with_dmn_decisions(vec![definition]);

    let resolved = package
        .find_dmn_decision(
            &DmnDecisionRef::new("Decision_invocation")
                .with_source_id("versioned-invocation-decision-20191111.dmn"),
        )
        .must("registered invocation decision lookup should succeed")
        .must("registered invocation decision should resolve");

    let invocation = resolved
        .invocation
        .as_ref()
        .must("invocation contract should be present");
    assert_eq!(invocation.invocation_id.as_deref(), Some("invocation_1"));
    assert_eq!(
        invocation
            .invoked_expression
            .as_ref()
            .map(|expression| expression.text.as_ref()),
        Some("scoreCard")
    );
    assert_eq!(invocation.bindings.len(), 1);
    assert_eq!(
        invocation.bindings[0]
            .parameter
            .as_ref()
            .and_then(|parameter| parameter.name.as_deref()),
        Some("age")
    );
}

#[test]
fn package_registered_dmn_decision_preserves_knowledge_requirement_contract() {
    let definition = parse_dmn_decision(&fixture_source(
        "versioned-local-required-knowledge-runtime-20191111.dmn",
        "versioned-local-required-knowledge-runtime-20191111.dmn",
    ))
    .must("required-knowledge invocation source should parse");
    let package = BpmnPackage::new("pkg_api", Vec::new()).with_dmn_decisions(vec![definition]);

    let resolved = package
        .find_dmn_decision(
            &DmnDecisionRef::new("Decision_required_knowledge_runtime")
                .with_source_id("versioned-local-required-knowledge-runtime-20191111.dmn"),
        )
        .must("registered required-knowledge decision lookup should succeed")
        .must("registered required-knowledge decision should resolve");

    assert_eq!(
        resolved.knowledge_requirements,
        vec![DmnKnowledgeRequirementReference::new(
            "requiredKnowledge",
            Some("#BKM_score_card"),
        )]
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

#[test]
fn package_registered_dmn_decision_service_resolves_deterministically() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "versioned-local-decision-service-runtime-20191111.dmn",
        "versioned-local-decision-service-runtime-20191111.dmn",
    ))
    .must("decision-service fixture should snapshot");
    let definition = DmnDecisionServiceDefinition::from_snapshot(
        "versioned-local-decision-service-runtime-20191111.dmn",
        snapshot
            .root
            .decision_services
            .first()
            .must("fixture should contain one top-level decision service"),
    );
    let package =
        BpmnPackage::new("pkg_api", Vec::new()).with_dmn_decision_services(vec![definition]);

    let resolved = package
        .find_dmn_decision_service(
            &DmnDecisionRef::new("DecisionService_credit")
                .with_source_id("versioned-local-decision-service-runtime-20191111.dmn"),
        )
        .must("registered decision-service lookup should stay deterministic")
        .must("registered decision service should resolve");

    assert_eq!(package.dmn_decision_services().len(), 1);
    assert_eq!(resolved.name.as_deref(), Some("Credit Decision Service"));
    assert_eq!(
        resolved.output_decisions,
        vec![DmnDecisionServiceReference::new(
            "outputDecision",
            Some("#Decision_approval"),
        )]
    );
}

#[test]
fn package_registered_dmn_import_preserves_resolution_metadata() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "unsupported-top-level-import-20191111.dmn",
        "unsupported-top-level-import-20191111.dmn",
    ))
    .must("import fixture should snapshot before executable parsing");
    let definition = DmnImportDefinition::from_snapshot(
        "unsupported-top-level-import-20191111.dmn",
        snapshot
            .root
            .imports
            .first()
            .must("fixture should contain one top-level import"),
    );
    let package = BpmnPackage::new("pkg_api", Vec::new()).with_dmn_imports(vec![definition]);

    let imports = package.dmn_imports_for_source("unsupported-top-level-import-20191111.dmn");

    assert_eq!(package.dmn_imports().len(), 1);
    assert_eq!(imports.len(), 1);
    assert_eq!(
        imports[0].source_id.as_ref(),
        "unsupported-top-level-import-20191111.dmn"
    );
    assert_eq!(imports[0].name.as_deref(), Some("Partner Services"));
    assert_eq!(
        imports[0].namespace.as_deref(),
        Some("https://example.com/dmn/partner-services")
    );
    assert_eq!(
        imports[0].location_uri.as_deref(),
        Some("partner-services.dmn")
    );
    assert_eq!(
        imports[0].import_type.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert!(
        package
            .dmn_imports_for_source("partner-services.dmn")
            .is_empty()
    );
}

#[test]
fn package_registered_dmn_import_lookup_is_source_scoped() {
    let package = BpmnPackage::new("pkg_api", Vec::new()).with_dmn_imports(vec![
        DmnImportDefinition::new(
            "customer.dmn",
            Some("Partner Services"),
            Some("https://example.com/dmn/customer-partner-services"),
            Some("customer-partner-services.dmn"),
            Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
        ),
        DmnImportDefinition::new(
            "risk.dmn",
            Some("Partner Services"),
            Some("https://example.com/dmn/risk-partner-services"),
            Some("risk-partner-services.dmn"),
            Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
        ),
    ]);

    let customer_import = package
        .find_dmn_import_by_name("customer.dmn", "Partner Services")
        .must("customer import lookup should be deterministic")
        .must("customer import should resolve");
    let risk_import = package
        .find_dmn_import_by_name("risk.dmn", "Partner Services")
        .must("risk import lookup should be deterministic")
        .must("risk import should resolve");

    assert_eq!(
        customer_import.namespace.as_deref(),
        Some("https://example.com/dmn/customer-partner-services")
    );
    assert_eq!(
        risk_import.namespace.as_deref(),
        Some("https://example.com/dmn/risk-partner-services")
    );
}

#[test]
fn package_registered_dmn_import_lookup_keeps_selectors_distinct() {
    let package = BpmnPackage::new("pkg_api", Vec::new()).with_dmn_imports(vec![
        DmnImportDefinition::new(
            "customer.dmn",
            Some("Partner Services"),
            Some("https://example.com/dmn/customer-partner-services"),
            Some("customer-partner-services.dmn"),
            Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
        ),
        DmnImportDefinition::new(
            "customer.dmn",
            Some("Customer Rules"),
            Some("https://example.com/dmn/customer-rules"),
            Some("customer-rules.dmn"),
            Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
        ),
    ]);

    let namespace_import = package
        .find_dmn_import_by_namespace(
            "customer.dmn",
            "https://example.com/dmn/customer-partner-services",
        )
        .must("namespace lookup should be deterministic")
        .must("namespace import should resolve");
    let location_import = package
        .find_dmn_import_by_location_uri("customer.dmn", "customer-rules.dmn")
        .must("location lookup should be deterministic")
        .must("location import should resolve");

    assert_eq!(namespace_import.name.as_deref(), Some("Partner Services"));
    assert_eq!(location_import.name.as_deref(), Some("Customer Rules"));
    assert!(
        package
            .find_dmn_import_by_namespace("customer.dmn", "Partner Services")
            .must("alias text should not match namespace selectors")
            .is_none()
    );
}

#[test]
fn package_registered_dmn_import_lookup_rejects_ambiguous_aliases() {
    let package = BpmnPackage::new("pkg_api", Vec::new()).with_dmn_imports(vec![
        DmnImportDefinition::new(
            "customer.dmn",
            Some("Partner Services"),
            Some("https://example.com/dmn/customer-partner-services-v1"),
            Some("customer-partner-services-v1.dmn"),
            Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
        ),
        DmnImportDefinition::new(
            "customer.dmn",
            Some("Partner Services"),
            Some("https://example.com/dmn/customer-partner-services-v2"),
            Some("customer-partner-services-v2.dmn"),
            Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
        ),
    ]);

    let error = package
        .find_dmn_import_by_name("customer.dmn", "Partner Services")
        .must_err("duplicate aliases in one declaring source should be ambiguous");

    assert_eq!(
        error,
        BpmnEngineError::AmbiguousDmnImportReference {
            source_id: "customer.dmn".to_string(),
            selector_kind: "name",
            selector_value: "Partner Services".to_string(),
            count: 2,
        }
    );
}

#[test]
fn package_registered_dmn_source_root_preserves_namespace_metadata() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "simple-unique-eligibility.dmn",
        "simple-unique-eligibility.dmn",
    ))
    .must("simple fixture should snapshot");
    let definition =
        DmnSourceDefinition::from_root_snapshot("simple-unique-eligibility.dmn", &snapshot.root);
    let package =
        BpmnPackage::new("pkg_api", Vec::new()).with_dmn_source_definitions(vec![definition]);

    let source = package
        .find_dmn_source_definition_by_namespace("http://example.com/dmn")
        .must("namespace lookup should be deterministic")
        .must("source root should resolve");

    assert_eq!(package.dmn_source_definitions().len(), 1);
    assert_eq!(source.source_id.as_ref(), "simple-unique-eligibility.dmn");
    assert_eq!(source.definitions_id.as_deref(), Some("Definitions_loan"));
    assert_eq!(source.name.as_deref(), Some("Loan DRD"));
    assert_eq!(source.namespace.as_deref(), Some("http://example.com/dmn"));
    assert_eq!(source.model_version_hint.as_deref(), Some("20191111"));
}

#[test]
fn package_registered_dmn_source_root_namespace_lookup_does_not_use_source_id() {
    let package = BpmnPackage::new("pkg_api", Vec::new())
        .with_dmn_imports(vec![DmnImportDefinition::new(
            "customer.dmn",
            Some("Partner Services"),
            Some("https://example.com/dmn/partner-services"),
            Some("partner-services.dmn"),
            Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
        )])
        .with_dmn_source_definitions(vec![DmnSourceDefinition::new(
            "partner-services.dmn",
            Some("Definitions_partner_services"),
            Some("Partner Services"),
            Some("https://example.com/dmn/partner-services"),
            Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
            Some("1.3"),
        )]);

    let imported_namespace = package
        .find_dmn_import_by_namespace("customer.dmn", "https://example.com/dmn/partner-services")
        .must("import namespace lookup should be deterministic")
        .must("import should resolve")
        .namespace
        .as_deref()
        .must("import should preserve target namespace");
    let source = package
        .find_dmn_source_definition_by_namespace(imported_namespace)
        .must("source namespace lookup should be deterministic")
        .must("source root should resolve by namespace");

    assert_eq!(source.source_id.as_ref(), "partner-services.dmn");
    assert!(
        package
            .find_dmn_source_definition_by_namespace("partner-services.dmn")
            .must("source id text should not match namespace lookup")
            .is_none()
    );
}

#[test]
fn package_registered_dmn_source_root_rejects_ambiguous_namespaces() {
    let package = BpmnPackage::new("pkg_api", Vec::new()).with_dmn_source_definitions(vec![
        DmnSourceDefinition::new(
            "partner-v1.dmn",
            Some("Definitions_partner_services_v1"),
            Some("Partner Services v1"),
            Some("https://example.com/dmn/partner-services"),
            Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
            Some("1.3"),
        ),
        DmnSourceDefinition::new(
            "partner-v2.dmn",
            Some("Definitions_partner_services_v2"),
            Some("Partner Services v2"),
            Some("https://example.com/dmn/partner-services"),
            Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
            Some("1.3"),
        ),
    ]);

    let error = package
        .find_dmn_source_definition_by_namespace("https://example.com/dmn/partner-services")
        .must_err("duplicate source namespaces should be ambiguous");

    assert_eq!(
        error,
        BpmnEngineError::AmbiguousDmnSourceNamespace {
            namespace: "https://example.com/dmn/partner-services".to_string(),
            count: 2,
        }
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
