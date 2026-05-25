use crate::public_api::dmn::support::{dmn_import, dmn_source};
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::{BpmnEngineError, BpmnPackage};

#[test]
fn package_registered_dmn_import_source_binding_rejects_ambiguous_namespaces() {
    let package = package_with_ambiguous_partner_services_namespace();
    let dmn_import = &package.dmn_imports()[0];
    let error = package
        .resolve_dmn_import_source(dmn_import)
        .must_err("duplicate target namespaces should be ambiguous");
    assert_partner_services_namespace_is_ambiguous(&error);
}

#[test]
fn package_registered_dmn_import_source_binding_report_marks_bound_and_unbound_imports() {
    let package = BpmnPackage::new("pkg_api", Vec::new())
        .with_dmn_imports(vec![
            dmn_import(
                "customer.dmn",
                Some("Partner Services"),
                Some("https://example.com/dmn/partner-services"),
                Some("partner-services.dmn"),
                Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
            ),
            dmn_import(
                "customer.dmn",
                Some("External Rules"),
                Some("https://example.com/dmn/external-rules"),
                Some("external-rules.dmn"),
                Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
            ),
            dmn_import(
                "customer.dmn",
                Some("Unnamed Namespace"),
                None::<&str>,
                Some("unnamed-namespace.dmn"),
                Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
            ),
        ])
        .with_dmn_source_definitions(vec![dmn_source(
            "partner-services.dmn",
            Some("Definitions_partner_services"),
            Some("Partner Services"),
            Some("https://example.com/dmn/partner-services"),
            Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
            Some("20191111"),
        )]);
    let bindings = package
        .dmn_import_source_bindings()
        .must("import binding report should be deterministic");
    assert_eq!(bindings.len(), 3);
    assert!(bindings[0].is_bound());
    assert_eq!(
        bindings[0]
            .source_definition
            .as_ref()
            .map(|source| source.source_id.as_ref()),
        Some("partner-services.dmn")
    );
    assert!(!bindings[1].is_bound());
    assert_eq!(
        bindings[1].dmn_import.namespace.as_deref(),
        Some("https://example.com/dmn/external-rules")
    );
    assert!(!bindings[2].is_bound());
    assert_eq!(bindings[2].dmn_import.namespace, None);
}

#[test]
fn package_registered_dmn_import_source_binding_report_rejects_ambiguous_targets() {
    let package = package_with_ambiguous_partner_services_namespace();
    let error = package
        .dmn_import_source_bindings()
        .must_err("binding report should reject ambiguous target namespaces");
    assert_partner_services_namespace_is_ambiguous(&error);
}

#[test]
fn package_registered_dmn_source_root_rejects_ambiguous_namespaces() {
    let package = BpmnPackage::new("pkg_api", Vec::new()).with_dmn_source_definitions(vec![
        partner_services_source(
            "partner-v1.dmn",
            "Definitions_partner_services_v1",
            "v1",
            "1.3",
        ),
        partner_services_source(
            "partner-v2.dmn",
            "Definitions_partner_services_v2",
            "v2",
            "1.3",
        ),
    ]);
    let error = package
        .find_dmn_source_definition_by_namespace("https://example.com/dmn/partner-services")
        .must_err("duplicate source namespaces should be ambiguous");
    assert_partner_services_namespace_is_ambiguous(&error);
}

fn package_with_ambiguous_partner_services_namespace() -> BpmnPackage {
    BpmnPackage::new("pkg_api", Vec::new())
        .with_dmn_imports(vec![dmn_import(
            "customer.dmn",
            Some("Partner Services"),
            Some("https://example.com/dmn/partner-services"),
            Some("partner-services.dmn"),
            Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
        )])
        .with_dmn_source_definitions(vec![
            partner_services_source(
                "partner-v1.dmn",
                "Definitions_partner_services_v1",
                "v1",
                "20191111",
            ),
            partner_services_source(
                "partner-v2.dmn",
                "Definitions_partner_services_v2",
                "v2",
                "20191111",
            ),
        ])
}

fn partner_services_source(
    source_id: &str,
    definitions_id: &str,
    partner_version: &str,
    model_version_hint: &str,
) -> xiuxian_qianji_bpmn_engine::DmnSourceDefinition {
    let name = format!("Partner Services {partner_version}");
    dmn_source(
        source_id,
        Some(definitions_id),
        Some(name.as_str()),
        Some("https://example.com/dmn/partner-services"),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/"),
        Some(model_version_hint),
    )
}

fn assert_partner_services_namespace_is_ambiguous(error: &BpmnEngineError) {
    assert_eq!(
        error,
        &BpmnEngineError::AmbiguousDmnSourceNamespace {
            namespace: "https://example.com/dmn/partner-services".to_string(),
            count: 2,
        }
    );
}
