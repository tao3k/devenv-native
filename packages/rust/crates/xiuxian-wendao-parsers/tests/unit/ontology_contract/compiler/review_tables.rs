use xiuxian_wendao_parsers::compile_org_ontology_authoring_document;

use crate::ontology_contract::support::{AUTHORING_SCHEMA, assert_valid, compile_schema};

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
