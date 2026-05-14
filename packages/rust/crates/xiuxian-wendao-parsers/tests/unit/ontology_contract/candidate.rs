use serde_json::json;

use super::support::{CANDIDATE_SCHEMA, assert_invalid, assert_valid, compile_schema};

#[test]
fn ontology_candidate_contract_accepts_valid_candidate_shape() {
    let schema = compile_schema(CANDIDATE_SCHEMA);
    let instance = json!({
        "schema": "xiuxian_wendao.ontology_candidate.v0.draft",
        "candidateId": "candidate:architecture-decision",
        "candidateKind": "object_type",
        "lifecycleState": "candidate",
        "source": {
            "authoringDocumentId": "org-authoring:software-engineering",
            "sectionId": "section:architecture-decision",
            "sourcePath": "wendao-episteme/ontology/software_engineering.org",
            "sourceHash": "blake3:authoring",
            "sourceSpan": {
                "startLine": 1,
                "endLine": 18
            }
        },
        "confidence": {
            "score": 0.95,
            "source": "parser_contract"
        },
        "payload": {
            "apiName": "ArchitectureDecision",
            "stableId": "software.architecture_decision"
        },
        "evidence": [
            {
                "evidenceId": "evidence:org-section",
                "kind": "org_authoring_section",
                "text": "Org heading path and property drawer declare this object type.",
                "sourceSpan": {
                    "startLine": 1,
                    "endLine": 18
                }
            }
        ]
    });

    assert_valid(&schema, &instance);
}

#[test]
fn ontology_candidate_contract_rejects_unknown_candidate_kind() {
    let schema = compile_schema(CANDIDATE_SCHEMA);
    let instance = json!({
        "schema": "xiuxian_wendao.ontology_candidate.v0.draft",
        "candidateId": "candidate:unknown",
        "candidateKind": "route_type",
        "lifecycleState": "candidate",
        "source": {
            "sourcePath": "traces/unknown.org",
            "sourceHash": "blake3:unknown",
            "sourceSpan": {
                "startLine": 1,
                "endLine": 2
            }
        },
        "confidence": {
            "score": 0.4,
            "source": "llm_candidate"
        },
        "payload": {
            "apiName": "Unknown"
        },
        "evidence": [
            {
                "evidenceId": "evidence:unknown",
                "kind": "org_trace_section",
                "text": "Unknown candidate kind should fail."
            }
        ]
    });

    assert_invalid(&schema, &instance);
}
