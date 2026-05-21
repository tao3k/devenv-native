use serde_json::json;
use xiuxian_wendao_parsers::{
    ORG_PROP_INVALID_CONFIDENCE, ORG_PROP_INVALID_ENUM, ORG_PROP_INVALID_SHA256,
    ORG_PROP_INVALID_UUID, ORG_PROP_MISSING_REQUIRED, ORG_PROP_UNKNOWN_PROPERTY,
    compile_org_ontology_authoring_document, compile_org_reasoning_property_records,
    validate_org_reasoning_properties,
};

use super::support::{PROPERTY_SCHEMA, assert_invalid, assert_valid, compile_schema};

#[test]
fn org_reasoning_property_schema_accepts_compiled_mapping_record() {
    let schema = compile_schema(PROPERTY_SCHEMA);
    let document = compile_property_fixture(
        "018f4a0d-3df0-7a8f-9f1d-000000000001",
        "ontology_mapping",
        &[
            (":ONTOLOGY_KIND:", "dataset_mapping"),
            (":STATUS:", "candidate"),
            (":PROMOTION_STATE:", "validated"),
            (":CONFIDENCE:", "0.98"),
            (
                ":SOURCE_SHA256:",
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ),
        ],
    );
    let records = compile_org_reasoning_property_records(&document);
    let instance = match serde_json::to_value(&records[0]) {
        Ok(instance) => instance,
        Err(error) => panic!("property record should serialize: {error}"),
    };

    assert_eq!(records.len(), 1);
    assert_valid(&schema, &instance);
    assert!(validate_org_reasoning_properties(&document).is_empty());
}

#[test]
fn org_reasoning_property_schema_reports_invalid_uuid() {
    let document = compile_property_fixture(
        "ltc.mapping.policy_city.001",
        "ontology_mapping",
        &[
            (":ONTOLOGY_KIND:", "dataset_mapping"),
            (":PROMOTION_STATE:", "candidate"),
        ],
    );
    let diagnostics = validate_org_reasoning_properties(&document);

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == ORG_PROP_INVALID_UUID && diagnostic.property.as_deref() == Some("ID")
    }));
}

#[test]
fn org_reasoning_property_schema_reports_unknown_property() {
    let document = compile_property_fixture(
        "018f4a0d-3df0-7a8f-9f1d-000000000002",
        "ontology_mapping",
        &[
            (":ONTOLOGY_KIND:", "dataset_mapping"),
            (":PROMOTION_STATE:", "candidate"),
            (":FREEFORM_THOUGHT:", "looks useful"),
        ],
    );
    let diagnostics = validate_org_reasoning_properties(&document);

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == ORG_PROP_UNKNOWN_PROPERTY
            && diagnostic.property.as_deref() == Some("FREEFORM_THOUGHT")
    }));
}

#[test]
fn org_reasoning_property_schema_reports_kind_specific_required_properties() {
    let document = compile_property_fixture(
        "018f4a0d-3df0-7a8f-9f1d-000000000003",
        "evidence_summary",
        &[(":ONTOLOGY_KIND:", "dataset_mapping")],
    );
    let diagnostics = validate_org_reasoning_properties(&document);

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == ORG_PROP_MISSING_REQUIRED
            && diagnostic.property.as_deref() == Some("SOURCE_HANDLE")
    }));
}

#[test]
fn org_reasoning_property_schema_reports_invalid_enum_confidence_and_hash() {
    let document = compile_property_fixture(
        "018f4a0d-3df0-7a8f-9f1d-000000000004",
        "ontology_mapping",
        &[
            (":ONTOLOGY_KIND:", "dataset_mapping"),
            (":PROMOTION_STATE:", "maybe"),
            (":CONFIDENCE:", "1.7"),
            (":SOURCE_SHA256:", "not-a-hash"),
        ],
    );
    let diagnostics = validate_org_reasoning_properties(&document);
    let codes = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect::<Vec<_>>();

    assert!(codes.contains(&ORG_PROP_INVALID_ENUM));
    assert!(codes.contains(&ORG_PROP_INVALID_CONFIDENCE));
    assert!(codes.contains(&ORG_PROP_INVALID_SHA256));
}

#[test]
fn org_reasoning_property_json_schema_rejects_unknown_property() {
    let schema = compile_schema(PROPERTY_SCHEMA);
    let instance = json!({
        "schema": "xiuxian_wendao.org_reasoning_property.v0.draft",
        "documentId": "org-authoring:test",
        "sourcePath": "wendao-episteme/ontology/test.org",
        "sourceHash": "blake3:test",
        "sectionId": "018f4a0d-3df0-7a8f-9f1d-000000000005",
        "headingPath": ["Test"],
        "properties": {
            "ID": "018f4a0d-3df0-7a8f-9f1d-000000000005",
            "WENDAO_KIND": "ontology_mapping",
            "PROMOTION_STATE": "candidate",
            "FREEFORM_THOUGHT": "not schema governed"
        },
        "sourceSpan": {
            "startLine": 1,
            "startColumn": 1,
            "endLine": 8,
            "endColumn": 1
        }
    });

    assert_invalid(&schema, &instance);
}

fn compile_property_fixture(
    id: &str,
    wendao_kind: &str,
    extra_properties: &[(&str, &str)],
) -> xiuxian_wendao_parsers::OrgOntologyAuthoringDocument {
    let mut content = String::from(
        ":PROPERTIES:\n:ID: org-authoring:property-schema\n:END:\n\
         #+TITLE: Property Schema\n\n\
         * TODO Reasoning Property Record\n\
         :PROPERTIES:\n",
    );
    content.push_str(format!(":ID: {id}\n:WENDAO_KIND: {wendao_kind}\n").as_str());
    for (key, value) in extra_properties {
        content.push_str(format!("{key} {value}\n").as_str());
    }
    content.push_str(":END:\nBody.\n");

    match compile_org_ontology_authoring_document(&content, "wendao-episteme/ontology/test.org") {
        Ok(document) => document,
        Err(error) => panic!("property schema fixture should compile: {error}"),
    }
}
