use xiuxian_wendao_parsers::{OrgOntologyAuthoringError, compile_org_ontology_authoring_document};

#[test]
fn org_ontology_authoring_compiler_rejects_untyped_org_sections() {
    let Err(error) = compile_org_ontology_authoring_document(
        "* Untyped Section\nBody.\n",
        "wendao-episteme/ontology/broken.org",
    ) else {
        panic!("missing ontology kind must fail before schema validation");
    };

    assert!(matches!(
        error,
        OrgOntologyAuthoringError::EmptyAuthoringDocument
    ));
}
