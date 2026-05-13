use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    DmnImportDefinition, DmnImportDefinitionInput, DmnSourceDefinition, DmnSourceDefinitionInput,
    DmnSourceFile,
};

pub(crate) fn fixture_source(source_id: &str, fixture_name: &str) -> DmnSourceFile {
    let path = format!(
        "{}/tests/fixtures/dmn/{fixture_name}",
        env!("CARGO_MANIFEST_DIR")
    );
    let contents = std::fs::read_to_string(path).must("fixture should be readable");
    DmnSourceFile::new(source_id, contents)
}

pub(crate) fn dmn_import(
    source_id: &str,
    name: Option<&str>,
    namespace: Option<&str>,
    location_uri: Option<&str>,
    import_type: Option<&str>,
) -> DmnImportDefinition {
    DmnImportDefinition::new(DmnImportDefinitionInput {
        source_id,
        name,
        namespace,
        location_uri,
        import_type,
    })
}

pub(crate) fn dmn_source(
    source_id: &str,
    definitions_id: Option<&str>,
    name: Option<&str>,
    namespace: Option<&str>,
    model_namespace_uri: Option<&str>,
    model_version_hint: Option<&str>,
) -> DmnSourceDefinition {
    DmnSourceDefinition::new(DmnSourceDefinitionInput {
        source_id: source_id.into(),
        definitions_id: definitions_id.map(Into::into),
        name,
        namespace,
        model_namespace_uri: model_namespace_uri.map(Into::into),
        model_version_hint,
    })
}
