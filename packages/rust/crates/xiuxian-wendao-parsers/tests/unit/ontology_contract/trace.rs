use serde_json::json;

use super::support::{TRACE_SCHEMA, assert_valid, compile_schema};

#[test]
fn org_trace_projection_contract_accepts_compiled_trace_shape() {
    let schema = compile_schema(TRACE_SCHEMA);
    let instance = json!({
        "schema": "xiuxian_wendao.org_trace_projection.v0.draft",
        "traceId": "trace:ontology-authoring-review",
        "sourcePath": "traces/ontology_review.org",
        "sourceHash": "blake3:trace",
        "sections": [
            {
                "traceSectionId": "trace-section:review-link",
                "path": ["Review", "Link Type"],
                "state": "done",
                "tags": ["ontology", "validation"],
                "properties": {
                    "CANDIDATE_ID": "candidate:depends_on",
                    "CONFIDENCE": "0.82"
                },
                "planning": {
                    "scheduled": "2026-05-13"
                },
                "tables": [
                    {
                        "name": "Findings",
                        "purpose": "validation_findings",
                        "columns": ["kind", "message"],
                        "rows": [
                            {
                                "kind": "relation_endpoint",
                                "message": "source and target object types exist"
                            }
                        ],
                        "sourceSpan": {
                            "startLine": 9,
                            "endLine": 11
                        }
                    }
                ],
                "sourceSpan": {
                    "startLine": 1,
                    "endLine": 11
                }
            }
        ]
    });

    assert_valid(&schema, &instance);
}
