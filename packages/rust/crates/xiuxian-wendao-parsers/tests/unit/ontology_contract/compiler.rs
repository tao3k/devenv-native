use xiuxian_wendao_parsers::{OrgOntologyAuthoringError, compile_org_ontology_authoring_document};

use super::support::{AUTHORING_SCHEMA, assert_valid, compile_schema};

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

    let document = compile_org_ontology_authoring_document(
        content,
        "wendao-episteme/ontology/software_engineering.org",
    )
    .expect("Org authoring fixture should compile");
    let instance = serde_json::to_value(&document).expect("DTO should serialize");

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

#[test]
fn org_ontology_authoring_compiler_projects_dataset_mapping_tables_and_sql_artifacts() {
    let schema = compile_schema(AUTHORING_SCHEMA);
    let content = concat!(
        ":PROPERTIES:\n",
        ":ID: org-authoring:healthcare-dataset\n",
        ":END:\n",
        "#+TITLE: Healthcare Dataset Mapping\n",
        "\n",
        "* DONE Healthcare Synthetic Mapping :ontology:mapping:\n",
        ":PROPERTIES:\n",
        ":ID: section:healthcare-synthetic-mapping\n",
        ":ONTOLOGY_KIND: dataset_mapping\n",
        ":STATUS: accepted\n",
        ":DOMAIN: episteme://30_Healthcare\n",
        ":MAPPING_ID: healthcare.synthetic_care_delivery.v1\n",
        ":END:\n",
        "| source_table | required_columns | ontology_role |\n",
        "| raw_patients | patient_id, patient_name | Patient object observations |\n",
        "\n",
        "| source_table | source_key | ontology_object_type | rdf_class | display_name |\n",
        "| raw_patients | patient_id | Patient | https://wendao.ai/ontology/healthcare#Patient | patient_name |\n",
        "\n",
        "| source_table | source_key | target_key | predicate | purpose |\n",
        "| raw_encounters | patient_id | encounter_id | https://wendao.ai/ontology/healthcare#hasEncounter | Patient.encounters |\n",
        "\n",
        "| evidence_id | source_table | source_key | review_state | decision |\n",
        "| evidence:raw_patients:P001 | raw_patients | P001 | accepted | patient source key is stable |\n",
        "\n",
        "#+BEGIN_SRC sql :purpose mapping\n",
        "SELECT * FROM ontology_object_observation;\n",
        "#+END_SRC\n",
    );

    let document = compile_org_ontology_authoring_document(
        content,
        "wendao-episteme/ontology/30_Healthcare/mappings/healthcare_dataset_mapping.org",
    )
    .expect("dataset mapping Org fixture should compile");
    let instance = serde_json::to_value(&document).expect("DTO should serialize");
    let section = &document.sections[0];
    let table_kinds = section
        .tables
        .iter()
        .map(|table| table.kind.as_str())
        .collect::<Vec<_>>();

    assert_eq!(section.authoring_kind, "dataset_mapping");
    assert_eq!(section.lifecycle_state, "accepted");
    assert_eq!(
        table_kinds,
        [
            "dataset_columns",
            "object_mapping",
            "link_mapping",
            "mapping_evidence"
        ]
    );
    assert_eq!(section.embedded_artifacts.len(), 1);
    assert_eq!(section.embedded_artifacts[0].language, "sql");
    assert_eq!(section.embedded_artifacts[0].purpose, "mapping");
    assert!(
        section
            .tables
            .iter()
            .all(|table| table.source_span.is_some())
    );
    assert!(section.embedded_artifacts[0].source_span.is_some());
    assert_valid(&schema, &instance);
}

#[test]
fn org_ontology_authoring_compiler_rejects_untyped_org_sections() {
    let error = compile_org_ontology_authoring_document(
        "* Untyped Section\nBody.\n",
        "wendao-episteme/ontology/broken.org",
    )
    .expect_err("missing ontology kind must fail before schema validation");

    assert!(matches!(
        error,
        OrgOntologyAuthoringError::MissingAuthoringKind { .. }
    ));
}
