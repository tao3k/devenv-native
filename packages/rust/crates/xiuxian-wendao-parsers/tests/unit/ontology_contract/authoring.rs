use serde_json::json;

use super::support::{AUTHORING_SCHEMA, assert_valid, compile_schema};

#[test]
fn org_ontology_authoring_contract_accepts_compiled_org_dto_shape() {
    let schema = compile_schema(AUTHORING_SCHEMA);
    let instance = json!({
        "schema": "xiuxian_wendao.org_ontology_authoring.v0.draft",
        "documentId": "org-authoring:software-engineering",
        "sourcePath": "wendao-episteme/ontology/software_engineering.org",
        "sourceHash": "blake3:authoring",
        "sections": [
            {
                "sectionId": "section:architecture-decision",
                "headingPath": ["Software Engineering", "Architecture Decision"],
                "level": 2,
                "title": "Architecture Decision",
                "authoringKind": "object_type",
                "lifecycleState": "candidate",
                "tags": ["software", "object-type"],
                "properties": {
                    "STABLE_ID": "software.architecture_decision",
                    "API_NAME": "ArchitectureDecision",
                    "OWNER": "wendao-episteme"
                },
                "tables": [
                    {
                        "name": "Fields",
                        "kind": "object_fields",
                        "columns": ["name", "type", "required"],
                        "rows": [
                            {
                                "name": "id",
                                "type": "string",
                                "required": "true"
                            }
                        ],
                        "sourceSpan": {
                            "startLine": 8,
                            "endLine": 10
                        }
                    }
                ],
                "embeddedArtifacts": [
                    {
                        "language": "rdf",
                        "purpose": "preview",
                        "contentHash": "blake3:rdf-preview",
                        "sourceSpan": {
                            "startLine": 12,
                            "endLine": 18
                        }
                    }
                ],
                "sourceSpan": {
                    "startLine": 1,
                    "endLine": 18
                }
            }
        ]
    });

    assert_valid(&schema, &instance);
}
