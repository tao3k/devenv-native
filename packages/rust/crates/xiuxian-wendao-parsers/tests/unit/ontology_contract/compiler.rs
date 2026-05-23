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

#[test]
fn org_ontology_authoring_compiler_projects_promotion_review_table() {
    let schema = compile_schema(AUTHORING_SCHEMA);
    let content = concat!(
        ":PROPERTIES:\n",
        ":ID: org-authoring:promotion-review\n",
        ":END:\n",
        "#+TITLE: Private Ontology Promotion Review Packet\n",
        "\n",
        "* TODO Promotion review packet\n",
        ":PROPERTIES:\n",
        ":ID: section:promotion-review\n",
        ":ONTOLOGY_KIND: dataset_mapping\n",
        ":LIFECYCLE_STATE: review\n",
        ":DOMAIN: episteme://private/medical-episteme/10_LongTermCare\n",
        ":END:\n",
        "| record_id | record_kind | review_decision | promotion_decision | reviewer_id |\n",
        "| candidate.term | ontology_candidate.object_term | ready_for_review | pending_review | reviewer.example |\n",
    );

    let document = match compile_org_ontology_authoring_document(
        content,
        "medical-episteme/runs/ontology-generation/ltc_ontology_seed_20260520/promotion_review.org",
    ) {
        Ok(document) => document,
        Err(error) => panic!("promotion review Org fixture should compile: {error}"),
    };
    let instance = match serde_json::to_value(&document) {
        Ok(instance) => instance,
        Err(error) => panic!("DTO should serialize: {error}"),
    };
    let section = &document.sections[0];

    assert_eq!(section.authoring_kind, "dataset_mapping");
    assert_eq!(section.lifecycle_state, "review");
    assert_eq!(section.tables.len(), 1);
    assert_eq!(section.tables[0].kind, "promotion_review");
    assert_eq!(section.tables[0].name, "Promotion Review");
    assert_valid(&schema, &instance);
}

#[test]
fn org_ontology_authoring_compiler_projects_instance_relation_review_table() {
    let schema = compile_schema(AUTHORING_SCHEMA);
    let content = concat!(
        ":PROPERTIES:\n",
        ":ID: org-authoring:instance-relation-review\n",
        ":END:\n",
        "#+TITLE: Private Instance Relation Review Ledger\n",
        "\n",
        "* TODO Evidence-backed relation review\n",
        ":PROPERTIES:\n",
        ":ID: section:instance-relation-review\n",
        ":ONTOLOGY_KIND: dataset_mapping\n",
        ":LIFECYCLE_STATE: review\n",
        ":DOMAIN: episteme://private/medical-episteme/10_LongTermCare\n",
        ":END:\n",
        "| relation_id | source_object_id | predicate | target_object_id | evidence_id | review_decision | promotion_decision | reviewer_id |\n",
        "| ltc.relation.example | ltc.policy.example | ltc.applies_to_city | ltc.city.shanghai | evidence:docling | evidence_candidate | pending_review | reviewer.example |\n",
    );

    let document = match compile_org_ontology_authoring_document(
        content,
        "medical-episteme/ontology/10_LongTermCare/review_ledgers/instance_relation_review.org",
    ) {
        Ok(document) => document,
        Err(error) => panic!("instance relation review Org fixture should compile: {error}"),
    };
    let instance = match serde_json::to_value(&document) {
        Ok(instance) => instance,
        Err(error) => panic!("DTO should serialize: {error}"),
    };
    let section = &document.sections[0];

    assert_eq!(section.authoring_kind, "dataset_mapping");
    assert_eq!(section.lifecycle_state, "review");
    assert_eq!(section.tables.len(), 1);
    assert_eq!(section.tables[0].kind, "instance_relation_review");
    assert_eq!(section.tables[0].name, "Instance Relation Review");
    assert_valid(&schema, &instance);
}

#[test]
fn org_ontology_authoring_compiler_projects_object_instance_review_table() {
    let schema = compile_schema(AUTHORING_SCHEMA);
    let content = concat!(
        ":PROPERTIES:\n",
        ":ID: org-authoring:object-instance-review\n",
        ":END:\n",
        "#+TITLE: Private Object Instance Review Ledger\n",
        "\n",
        "* TODO Evidence-backed object review\n",
        ":PROPERTIES:\n",
        ":ID: section:object-instance-review\n",
        ":ONTOLOGY_KIND: dataset_mapping\n",
        ":LIFECYCLE_STATE: review\n",
        ":DOMAIN: episteme://private/medical-episteme/10_LongTermCare\n",
        ":END:\n",
        "| object_id | object_type | label | evidence_id | review_decision | promotion_decision | reviewer_id |\n",
        "| ltc.city.shanghai | ltc.pilot_city | 上海市 | evidence:docling | evidence_candidate | pending_review | reviewer.example |\n",
    );

    let document = match compile_org_ontology_authoring_document(
        content,
        "medical-episteme/ontology/10_LongTermCare/review_ledgers/object_instance_review.org",
    ) {
        Ok(document) => document,
        Err(error) => panic!("object instance review Org fixture should compile: {error}"),
    };
    let instance = match serde_json::to_value(&document) {
        Ok(instance) => instance,
        Err(error) => panic!("DTO should serialize: {error}"),
    };
    let section = &document.sections[0];

    assert_eq!(section.authoring_kind, "dataset_mapping");
    assert_eq!(section.lifecycle_state, "review");
    assert_eq!(section.tables.len(), 1);
    assert_eq!(section.tables[0].kind, "object_instance_review");
    assert_eq!(section.tables[0].name, "Object Instance Review");
    assert_valid(&schema, &instance);
}

#[test]
fn org_ontology_authoring_compiler_projects_nested_review_tables() {
    let schema = compile_schema(AUTHORING_SCHEMA);
    let content = concat!(
        ":PROPERTIES:\n",
        ":ID: org-authoring:nested-object-instance-review\n",
        ":END:\n",
        "#+TITLE: Private Nested Object Instance Review Ledger\n",
        "\n",
        "* TODO Evidence-backed object review\n",
        ":PROPERTIES:\n",
        ":ID: section:nested-object-instance-review\n",
        ":ONTOLOGY_KIND: dataset_mapping\n",
        ":LIFECYCLE_STATE: review\n",
        ":DOMAIN: episteme://private/medical-episteme/10_LongTermCare\n",
        ":END:\n",
        "\n",
        "** Object candidates\n",
        "\n",
        "| object_id | object_type | label | evidence_id | review_decision | promotion_decision | reviewer_id |\n",
        "| ltc.city.shanghai | ltc.pilot_city | 上海市 | evidence:docling | evidence_candidate | pending_review | reviewer.example |\n",
    );

    let document = match compile_org_ontology_authoring_document(
        content,
        "medical-episteme/ontology/10_LongTermCare/review_ledgers/nested_object_instance_review.org",
    ) {
        Ok(document) => document,
        Err(error) => panic!("nested object instance review Org fixture should compile: {error}"),
    };
    let instance = match serde_json::to_value(&document) {
        Ok(instance) => instance,
        Err(error) => panic!("DTO should serialize: {error}"),
    };
    let section = &document.sections[0];

    assert_eq!(section.authoring_kind, "dataset_mapping");
    assert_eq!(section.tables.len(), 1);
    assert_eq!(section.tables[0].kind, "object_instance_review");
    assert_eq!(section.tables[0].rows.len(), 1);
    assert_valid(&schema, &instance);
}

#[test]
fn org_ontology_authoring_compiler_projects_candidate_review_table() {
    let schema = compile_schema(AUTHORING_SCHEMA);
    let content = concat!(
        ":PROPERTIES:\n",
        ":ID: org-authoring:candidate-review\n",
        ":END:\n",
        "#+TITLE: Private Ontology Candidate Review Packet\n",
        "\n",
        "* TODO Candidate review packet\n",
        ":PROPERTIES:\n",
        ":ID: section:candidate-review\n",
        ":ONTOLOGY_KIND: dataset_mapping\n",
        ":LIFECYCLE_STATE: review\n",
        ":DOMAIN: episteme://private/medical-episteme/10_LongTermCare\n",
        ":END:\n",
        "| record_id | record_kind | review_decision | promotion_precondition_met |\n",
        "| candidate.term | ontology_candidate.object_term | ready_for_review | false |\n",
    );

    let document = match compile_org_ontology_authoring_document(
        content,
        "medical-episteme/runs/ontology-generation/ltc_ontology_seed_20260520/candidate_review.org",
    ) {
        Ok(document) => document,
        Err(error) => panic!("candidate review Org fixture should compile: {error}"),
    };
    let instance = match serde_json::to_value(&document) {
        Ok(instance) => instance,
        Err(error) => panic!("DTO should serialize: {error}"),
    };
    let section = &document.sections[0];

    assert_eq!(section.authoring_kind, "dataset_mapping");
    assert_eq!(section.lifecycle_state, "review");
    assert_eq!(section.tables.len(), 1);
    assert_eq!(section.tables[0].kind, "candidate_review");
    assert_eq!(section.tables[0].name, "Candidate Review");
    assert_valid(&schema, &instance);
}

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
