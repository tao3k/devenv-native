use super::{TEST_VALKEY_URL, unique_prefix};
use xiuxian_wendao::{LinkGraphSuggestedLinkRequest, valkey_suggested_link_log_with_valkey};

#[test]
fn test_suggested_link_log_rejects_invalid_payload() {
    let prefix = unique_prefix();
    let result = valkey_suggested_link_log_with_valkey(
        &LinkGraphSuggestedLinkRequest {
            source_id: String::new(),
            target_id: "docs/b.md".to_string(),
            relation: "related_to".to_string(),
            confidence: 0.4,
            evidence: "test".to_string(),
            agent_id: "wendao-agentic-architect".to_string(),
            created_at_unix: None,
        },
        TEST_VALKEY_URL,
        Some(&prefix),
        Some(10),
        None,
    );
    assert!(result.is_err());
}
