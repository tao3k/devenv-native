use crate::public_api::dmn::support::{dmn_import, dmn_source};
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::{BpmnEngineError, BpmnPackage};

#[test]
fn package_registered_dmn_import_lookup_is_source_scoped() {
    let package = BpmnPackage::new("pkg_api", Vec::new()).with_dmn_imports(vec![
        dmn_import(
            "customer.dmn",
            Some("Partner Services"),
            Some("https://example.com/dmn/customer-partner-services"),
            Some("customer-partner-services.dmn"),
            Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
        ),
        dmn_import(
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
        dmn_import(
            "customer.dmn",
            Some("Partner Services"),
            Some("https://example.com/dmn/customer-partner-services"),
            Some("customer-partner-services.dmn"),
            Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
        ),
        dmn_import(
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
        dmn_import(
            "customer.dmn",
            Some("Partner Services"),
            Some("https://example.com/dmn/customer-partner-services-v1"),
            Some("customer-partner-services-v1.dmn"),
            Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
        ),
        dmn_import(
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
            source_id: ("customer.dmn".to_string()).into(),
            selector_kind: "name",
            selector_value: "Partner Services".to_string(),
            count: 2,
        }
    );
}

#[test]
fn package_registered_dmn_source_root_namespace_lookup_does_not_use_source_id() {
    let package = BpmnPackage::new("pkg_api", Vec::new())
        .with_dmn_imports(vec![dmn_import(
            "customer.dmn",
            Some("Partner Services"),
            Some("https://example.com/dmn/partner-services"),
            Some("partner-services.dmn"),
            Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
        )])
        .with_dmn_source_definitions(vec![dmn_source(
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
fn package_registered_dmn_import_resolves_bundled_source_root_by_namespace() {
    let package = BpmnPackage::new("pkg_api", Vec::new())
        .with_dmn_imports(vec![dmn_import(
            "customer.dmn",
            Some("Partner Services"),
            Some("https://example.com/dmn/partner-services"),
            Some("partner-services.dmn"),
            Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
        )])
        .with_dmn_source_definitions(vec![dmn_source(
            "partner-services.dmn",
            Some("Definitions_partner_services"),
            Some("Partner Services"),
            Some("https://example.com/dmn/partner-services"),
            Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
            Some("20191111"),
        )]);
    let dmn_import = package
        .find_dmn_import_by_name("customer.dmn", "Partner Services")
        .must("import lookup should be deterministic")
        .must("import should resolve");
    let source = package
        .resolve_dmn_import_source(dmn_import)
        .must("import source binding should be deterministic")
        .must("import should bind to bundled source root");
    assert_eq!(source.source_id.as_ref(), "partner-services.dmn");
    assert_eq!(
        source.namespace.as_deref(),
        Some("https://example.com/dmn/partner-services")
    );
}

#[test]
fn package_registered_dmn_import_without_namespace_has_no_source_binding() {
    let package = BpmnPackage::new("pkg_api", Vec::new())
        .with_dmn_imports(vec![dmn_import(
            "customer.dmn",
            Some("Partner Services"),
            None::<&str>,
            Some("partner-services.dmn"),
            Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
        )])
        .with_dmn_source_definitions(vec![dmn_source(
            "partner-services.dmn",
            Some("Definitions_partner_services"),
            Some("Partner Services"),
            Some("https://example.com/dmn/partner-services"),
            Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
            Some("20191111"),
        )]);
    let dmn_import = package.dmn_imports()[0].clone();
    assert!(
        package
            .resolve_dmn_import_source(&dmn_import)
            .must("missing import namespace should not be ambiguous")
            .is_none()
    );
}
