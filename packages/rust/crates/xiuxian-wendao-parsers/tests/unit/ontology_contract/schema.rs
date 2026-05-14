use serde_json::json;

use super::support::{
    AUTHORING_SCHEMA, CANDIDATE_SCHEMA, TRACE_SCHEMA, assert_invalid, compile_schema,
};

#[test]
fn ontology_json_schema_contracts_compile() {
    for raw_schema in [AUTHORING_SCHEMA, TRACE_SCHEMA, CANDIDATE_SCHEMA] {
        compile_schema(raw_schema);
    }
}

#[test]
fn ontology_json_schema_contracts_reject_missing_source_spans() {
    let schema = compile_schema(AUTHORING_SCHEMA);
    let instance = json!({
        "schema": "xiuxian_wendao.org_ontology_authoring.v0.draft",
        "documentId": "org-authoring:broken",
        "sourcePath": "wendao-episteme/ontology/broken.org",
        "sourceHash": "blake3:broken",
        "sections": [
            {
                "sectionId": "section:missing-span",
                "headingPath": ["Broken"],
                "level": 1,
                "title": "Broken",
                "authoringKind": "object_type",
                "lifecycleState": "candidate",
                "properties": {}
            }
        ]
    });

    assert_invalid(&schema, &instance);
}

#[test]
fn org_ontology_authoring_contract_rejects_unknown_table_kind() {
    let schema = compile_schema(AUTHORING_SCHEMA);
    let instance = json!({
        "schema": "xiuxian_wendao.org_ontology_authoring.v0.draft",
        "documentId": "org-authoring:bad-dataset-mapping",
        "sourcePath": "wendao-episteme/ontology/30_Healthcare/mappings/bad.org",
        "sourceHash": "blake3:bad",
        "sections": [
            {
                "sectionId": "section:bad",
                "headingPath": ["Bad"],
                "level": 1,
                "title": "Bad",
                "authoringKind": "dataset_mapping",
                "lifecycleState": "candidate",
                "properties": {},
                "tables": [
                    {
                        "name": "Unsafe",
                        "kind": "raw_row_truth",
                        "columns": ["source_table"],
                        "rows": [],
                        "sourceSpan": {
                            "startLine": 4,
                            "endLine": 5
                        }
                    }
                ],
                "sourceSpan": {
                    "startLine": 1,
                    "endLine": 5
                }
            }
        ]
    });

    assert_invalid(&schema, &instance);
}
