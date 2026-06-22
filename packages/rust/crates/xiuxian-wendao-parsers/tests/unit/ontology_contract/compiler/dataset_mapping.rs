use xiuxian_wendao_parsers::compile_org_ontology_authoring_document;

use crate::ontology_contract::support::{AUTHORING_SCHEMA, assert_valid, compile_schema};

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

    let document = match compile_org_ontology_authoring_document(
        content,
        "wendao-episteme/ontology/30_Healthcare/mappings/healthcare_dataset_mapping.org",
    ) {
        Ok(document) => document,
        Err(error) => panic!("dataset mapping Org fixture should compile: {error}"),
    };
    let instance = match serde_json::to_value(&document) {
        Ok(instance) => instance,
        Err(error) => panic!("DTO should serialize: {error}"),
    };
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
