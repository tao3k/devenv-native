use xiuxian_wendao_core::{
    SemanticScopeProjectionStaleness, parse_semantic_scope_metadata_envelope_json,
    semantic_scope_metadata_envelope_to_vec,
};

#[test]
fn parses_semantic_scope_metadata_envelope() {
    let raw = serde_json::json!({
        "semanticScopeBundle": semantic_scope_bundle_value("fresh"),
        "semanticProjectionPolicyEvidence": {
            "policyId": "semantic_projection.required_refresh_targets",
            "status": "passed",
            "failingProjectionCount": 0,
            "message": "all active change-intent projection refresh targets are fresh",
            "projections": []
        }
    })
    .to_string();

    let envelope = parse_semantic_scope_metadata_envelope_json(&raw)
        .unwrap_or_else(|error| panic!("semantic-scope envelope should parse: {error}"));

    assert_eq!(envelope.bundle.task_id.as_deref(), Some("task.demo"));
    assert_eq!(envelope.bundle.objects[0].kind.as_str(), "component");
    assert_eq!(
        envelope
            .bundle
            .projection_staleness
            .as_ref()
            .map(SemanticScopeProjectionStaleness::as_str),
        Some("fresh")
    );
    assert_eq!(
        envelope
            .projection_policy_evidence
            .as_ref()
            .map(|policy| policy.status.as_str()),
        Some("passed")
    );

    let encoded = semantic_scope_metadata_envelope_to_vec(&envelope)
        .unwrap_or_else(|error| panic!("semantic-scope envelope should serialize: {error}"));
    assert!(!encoded.is_empty());
}

#[test]
fn parses_raw_semantic_scope_bundle_for_compatibility() {
    let raw = semantic_scope_bundle_value("stale").to_string();

    let envelope = parse_semantic_scope_metadata_envelope_json(&raw)
        .unwrap_or_else(|error| panic!("raw semantic-scope bundle should parse: {error}"));

    assert_eq!(envelope.bundle.task_id.as_deref(), Some("task.demo"));
    assert_eq!(envelope.bundle.change_intents[0].id, "change.demo");
    assert_eq!(envelope.projection_policy_evidence, None);
}

fn semantic_scope_bundle_value(projection_staleness: &str) -> serde_json::Value {
    serde_json::json!({
        "task_id": "task.demo",
        "requested_object_ids": ["component.demo", "task.demo"],
        "objects": [
            {
                "id": "component.demo",
                "kind": "component",
                "title": "Demo Component",
                "status": "active",
                "confidence": {
                    "score": 0.95,
                    "source": "verified"
                },
                "owners": [
                    {
                        "scope": "xiuxian-qianji",
                        "role": "semantic_scope_consumer"
                    }
                ],
                "provenance": {
                    "source": "docs/rfcs/demo.md",
                    "recorded_by": "test",
                    "recorded_at": "2026-05-05"
                },
                "verification": {
                    "required": ["cargo test -p xiuxian-qianji workdir_semantic"]
                },
                "relations": []
            },
            {
                "id": "task.demo",
                "kind": "task",
                "title": "Candidate Demo Task",
                "status": "candidate",
                "confidence": {
                    "score": 0.55,
                    "source": "llm_suggested"
                },
                "owners": [],
                "provenance": {
                    "source": "semantic/change-intents/demo-change.md",
                    "recorded_by": "test",
                    "recorded_at": "2026-05-05"
                },
                "verification": {
                    "required": ["cargo test -p xiuxian-qianji workdir_semantic"]
                },
                "relations": []
            }
        ],
        "relations": [
            {
                "source": "component.demo",
                "kind": "validates",
                "target": "task.demo"
            }
        ],
        "change_intents": [
            {
                "type": "semantic_change_intent",
                "id": "change.demo",
                "title": "Demo Change",
                "status": "active",
                "touched_objects": ["component.demo"],
                "changed_relations": [],
                "affected_invariants": ["task.demo"],
                "required_validations": ["cargo test -p xiuxian-qianji workdir_semantic"],
                "projections_to_refresh": ["llm_compression"],
                "candidate_suggestions": ["task.demo"]
            }
        ],
        "affected_invariants": ["task.demo"],
        "required_validations": ["cargo test -p xiuxian-qianji workdir_semantic"],
        "projection_revision": "semantic-scope-demo",
        "projection_source_revision": "blake3:demo",
        "projection_staleness": projection_staleness,
        "unresolved_ids": []
    })
}
