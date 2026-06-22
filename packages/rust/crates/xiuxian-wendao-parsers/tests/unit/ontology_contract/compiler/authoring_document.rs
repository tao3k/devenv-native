use xiuxian_wendao_parsers::compile_org_ontology_authoring_document;

use crate::ontology_contract::support::{AUTHORING_SCHEMA, assert_valid, compile_schema};

#[test]
fn org_ontology_authoring_compiler_projects_real_org_into_schema_valid_dto() {
    let schema = compile_schema(AUTHORING_SCHEMA);
    let content = concat!(
        ":PROPERTIES:\n",
        ":ID: org-authoring:software-engineering\n",
        ":END:\n",
        "#+TITLE: Software Engineering Ontology\n",
        "\n",
        "* TODO Software Engineering :ontology:software:\n",
        ":PROPERTIES:\n",
        ":ID: section:software-engineering\n",
        ":ONTOLOGY_KIND: domain\n",
        ":STATUS: candidate\n",
        ":OWNER: wendao-episteme\n",
        ":END:\n",
        "Domain shell.\n",
        "** DONE Architecture Decision :object_type:\n",
        ":PROPERTIES:\n",
        ":ID: section:architecture-decision\n",
        ":ONTOLOGY_KIND: object_type\n",
        ":STATUS: accepted\n",
        ":API_NAME: ArchitectureDecision\n",
        ":STABLE_ID: software.architecture_decision\n",
        ":END:\n",
        "Object type body.\n",
    );

    let document = match compile_org_ontology_authoring_document(
        content,
        "wendao-episteme/ontology/software_engineering.org",
    ) {
        Ok(document) => document,
        Err(error) => panic!("Org authoring fixture should compile: {error}"),
    };
    let instance = match serde_json::to_value(&document) {
        Ok(instance) => instance,
        Err(error) => panic!("DTO should serialize: {error}"),
    };

    assert_eq!(document.document_id, "org-authoring:software-engineering");
    assert_eq!(document.sections.len(), 2);
    assert_eq!(document.sections[0].authoring_kind, "domain");
    assert_eq!(document.sections[0].lifecycle_state, "candidate");
    assert_eq!(
        document.sections[1].heading_path,
        [
            "Software Engineering".to_string(),
            "Architecture Decision".to_string()
        ]
    );
    assert_eq!(document.sections[1].lifecycle_state, "accepted");
    assert!(
        document.sections[1]
            .tags
            .contains(&"object_type".to_string())
    );
    assert_valid(&schema, &instance);
}
