use std::collections::BTreeMap;
use std::path::PathBuf;

use xiuxian_wendao_core::{
    ContractFindingConfidence, ContractFindingSeverity, ContractKnowledgeBatch,
    ContractKnowledgeDecision, ContractKnowledgeEnvelope, KnowledgeCategory,
    WendaoContractFeedbackAdapter,
};

fn rest_failure_envelope() -> ContractKnowledgeEnvelope {
    ContractKnowledgeEnvelope {
        entry_id: "wendao-contracts::rest_docs::REST-R003::/api/search/definition".to_string(),
        suite_id: "wendao-contracts".to_string(),
        generated_at: "2026-03-17T00:00:00Z".to_string(),
        rule_id: "REST-R003".to_string(),
        pack_id: "rest_docs".to_string(),
        domain: "rest".to_string(),
        severity: ContractFindingSeverity::Error,
        decision: ContractKnowledgeDecision::Fail,
        confidence: ContractFindingConfidence::High,
        title: "[REST-R003] Incomplete response documentation".to_string(),
        content: "Summary: Missing error response description".to_string(),
        summary: "The endpoint is missing response descriptions.".to_string(),
        evidence_excerpt: Some(
            "Error response `500` is missing a non-empty description.".to_string(),
        ),
        why_it_matters: "Clients need explicit success and error coverage.".to_string(),
        remediation: "Document both success and error responses.".to_string(),
        good_example: Some(
            "Document `200` and `400` with short response descriptions.".to_string(),
        ),
        bad_example: Some("Expose statuses without any descriptions.".to_string()),
        source_path: Some(PathBuf::from(
            "packages/rust/crates/xiuxian-wendao/openapi.json",
        )),
        tags: vec![
            "contract_finding".to_string(),
            "domain:rest".to_string(),
            "pack:rest_docs".to_string(),
        ],
        metadata: BTreeMap::from([("trace_ids".to_string(), serde_json::json!(["trace-42"]))]),
    }
}

fn generated_rest_docs_batch() -> ContractKnowledgeBatch {
    let mut envelope = rest_failure_envelope();
    envelope.entry_id = "wendao-contracts::rest_docs::REST-R007::/documents".to_string();
    envelope.rule_id = "REST-R007".to_string();
    envelope.pack_id = "rest_docs".to_string();
    envelope.domain = "rest".to_string();
    envelope.decision = ContractKnowledgeDecision::Warn;
    envelope.severity = ContractFindingSeverity::Warning;
    envelope.title = "[REST-R007] Request schema is underspecified".to_string();
    envelope.tags = vec![
        "contract_finding".to_string(),
        "domain:rest".to_string(),
        "pack:rest_docs".to_string(),
        "rule:REST-R007".to_string(),
    ];

    ContractKnowledgeBatch {
        suite_id: "wendao-contracts".to_string(),
        generated_at: "2026-03-18T00:00:00Z".to_string(),
        entries: vec![envelope],
    }
}

fn generated_modularity_batch() -> ContractKnowledgeBatch {
    let mut envelope = rest_failure_envelope();
    envelope.entry_id = "wendao-contracts::modularity::MOD-R002::internal::state.rs".to_string();
    envelope.rule_id = "MOD-R002".to_string();
    envelope.pack_id = "modularity".to_string();
    envelope.domain = "architecture".to_string();
    envelope.decision = ContractKnowledgeDecision::Warn;
    envelope.severity = ContractFindingSeverity::Warning;
    envelope.title = "[MOD-R002] Module boundary warning".to_string();
    envelope.tags = vec![
        "contract_finding".to_string(),
        "domain:architecture".to_string(),
        "pack:modularity".to_string(),
        "rule:MOD-R002".to_string(),
    ];

    ContractKnowledgeBatch {
        suite_id: "wendao-contracts".to_string(),
        generated_at: "2026-03-18T00:00:00Z".to_string(),
        entries: vec![envelope],
    }
}

#[test]
fn contract_feedback_adapter_maps_failure_to_error_entry() {
    let envelope = rest_failure_envelope();

    let entry = WendaoContractFeedbackAdapter::knowledge_entry_from_envelope(&envelope);

    assert_eq!(entry.id, envelope.entry_id);
    assert_eq!(entry.title, envelope.title);
    assert_eq!(entry.content, envelope.content);
    assert_eq!(entry.category, KnowledgeCategory::Error);
    assert_eq!(
        entry.source.as_deref(),
        Some("packages/rust/crates/xiuxian-wendao/openapi.json")
    );
    assert!(entry.tags.iter().any(|tag| tag == "contract_feedback"));
    assert!(entry.tags.iter().any(|tag| tag == "decision:fail"));
    assert!(entry.tags.iter().any(|tag| tag == "category:error"));
    assert!(
        entry.metadata.get("evidence_excerpt").is_some_and(
            |value| value == "Error response `500` is missing a non-empty description."
        )
    );
    assert!(
        entry
            .metadata
            .get("trace_ids")
            .is_some_and(|value| value == &serde_json::json!(["trace-42"]))
    );
}

#[test]
fn contract_feedback_adapter_uses_pack_specific_non_failure_categories() {
    let mut envelope = rest_failure_envelope();
    envelope.decision = ContractKnowledgeDecision::Warn;
    envelope.severity = ContractFindingSeverity::Warning;
    envelope.pack_id = "modularity".to_string();
    envelope.domain = "architecture".to_string();

    let category = WendaoContractFeedbackAdapter::knowledge_category(&envelope);

    assert_eq!(category, KnowledgeCategory::Architecture);
}

#[test]
fn contract_feedback_adapter_maps_batches_to_entries() {
    let first = rest_failure_envelope();
    let mut second = rest_failure_envelope();
    second.entry_id = "wendao-contracts::multi_role_audit::AUDIT-R003::global".to_string();
    second.rule_id = "AUDIT-R003".to_string();
    second.pack_id = "multi_role_audit".to_string();
    second.domain = "audit".to_string();
    second.decision = ContractKnowledgeDecision::Warn;
    second.severity = ContractFindingSeverity::Warning;
    second.title = "[AUDIT-R003] Runtime drift warning".to_string();

    let batch = ContractKnowledgeBatch {
        suite_id: "wendao-contracts".to_string(),
        generated_at: "2026-03-17T00:00:00Z".to_string(),
        entries: vec![first, second],
    };

    let entries = WendaoContractFeedbackAdapter::knowledge_entries_from_batch(&batch);

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].category, KnowledgeCategory::Error);
    assert_eq!(entries[1].category, KnowledgeCategory::Workflow);
}

#[test]
fn contract_feedback_adapter_maps_generated_rest_docs_batch_to_reference_entries() {
    let batch = generated_rest_docs_batch();

    assert_eq!(batch.entries.len(), 1);
    assert_eq!(batch.entries[0].pack_id, "rest_docs");
    assert_eq!(batch.entries[0].decision, ContractKnowledgeDecision::Warn);

    let entries = WendaoContractFeedbackAdapter::knowledge_entries_from_batch(&batch);

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].category, KnowledgeCategory::Reference);
    assert!(entries[0].tags.iter().any(|tag| tag == "pack:rest_docs"));
    assert!(
        entries[0]
            .tags
            .iter()
            .any(|tag| tag == "category:reference")
    );
    assert!(
        entries[0]
            .metadata
            .get("rule_id")
            .is_some_and(|value| value == "REST-R007")
    );
}

#[test]
fn contract_feedback_adapter_maps_generated_modularity_batch_to_architecture_entries() {
    let batch = generated_modularity_batch();

    assert_eq!(batch.entries.len(), 1);
    assert_eq!(batch.entries[0].pack_id, "modularity");
    assert_eq!(batch.entries[0].decision, ContractKnowledgeDecision::Warn);

    let entries = WendaoContractFeedbackAdapter::knowledge_entries_from_batch(&batch);

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].category, KnowledgeCategory::Architecture);
    assert!(entries[0].tags.iter().any(|tag| tag == "pack:modularity"));
    assert!(
        entries[0]
            .tags
            .iter()
            .any(|tag| tag == "category:architecture")
    );
    assert!(
        entries[0]
            .metadata
            .get("rule_id")
            .is_some_and(|value| value == "MOD-R002")
    );
}
