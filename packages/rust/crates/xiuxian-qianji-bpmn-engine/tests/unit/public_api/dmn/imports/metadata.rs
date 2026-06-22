use crate::public_api::dmn::support::fixture_source;
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::{
    BpmnPackage, DmnImportDefinition, DmnSourceDefinition, snapshot_dmn_source,
};

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
