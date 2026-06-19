use super::{TEST_VALKEY_URL, clear_prefix, unique_prefix};
use xiuxian_wendao::{
    LinkGraphSuggestedLinkDecisionRequest, LinkGraphSuggestedLinkRequest,
    LinkGraphSuggestedLinkState, valkey_suggested_link_decide_with_valkey,
    valkey_suggested_link_log_with_valkey,
};

#[test]
fn test_suggested_link_decide_rejects_invalid_transition() -> Result<(), Box<dyn std::error::Error>>
{
    let prefix = unique_prefix();
    if clear_prefix(&prefix).is_err() {
        return Ok(());
    }

    let entry = valkey_suggested_link_log_with_valkey(
        &LinkGraphSuggestedLinkRequest {
            source_id: "docs/x.md".to_string(),
            target_id: "docs/y.md".to_string(),
            relation: "related_to".to_string(),
            confidence: 0.42,
            evidence: "bridge".to_string(),
            agent_id: "wendao-agentic-architect".to_string(),
            created_at_unix: Some(1_700_000_200.0),
        },
        TEST_VALKEY_URL,
        Some(&prefix),
        Some(20),
        None,
    )
    .map_err(|err| err.clone())?;

    let invalid = valkey_suggested_link_decide_with_valkey(
        &LinkGraphSuggestedLinkDecisionRequest {
            suggestion_id: entry.suggestion_id.clone(),
            target_state: LinkGraphSuggestedLinkState::Provisional,
            decided_by: "omega-gate".to_string(),
            reason: "no-op".to_string(),
            decided_at_unix: Some(1_700_000_220.0),
        },
        TEST_VALKEY_URL,
        Some(&prefix),
        Some(20),
        None,
    );
    assert!(invalid.is_err());

    let first_decision = valkey_suggested_link_decide_with_valkey(
        &LinkGraphSuggestedLinkDecisionRequest {
            suggestion_id: entry.suggestion_id.clone(),
            target_state: LinkGraphSuggestedLinkState::Rejected,
            decided_by: "omega-gate".to_string(),
            reason: "insufficient evidence".to_string(),
            decided_at_unix: Some(1_700_000_230.0),
        },
        TEST_VALKEY_URL,
        Some(&prefix),
        Some(20),
        None,
    )
    .map_err(|err| err.clone())?;
    assert_eq!(
        first_decision.suggestion.promotion_state,
        LinkGraphSuggestedLinkState::Rejected
    );

    let second_decision = valkey_suggested_link_decide_with_valkey(
        &LinkGraphSuggestedLinkDecisionRequest {
            suggestion_id: entry.suggestion_id,
            target_state: LinkGraphSuggestedLinkState::Promoted,
            decided_by: "omega-gate".to_string(),
            reason: "retry".to_string(),
            decided_at_unix: Some(1_700_000_240.0),
        },
        TEST_VALKEY_URL,
        Some(&prefix),
        Some(20),
        None,
    );
    assert!(second_decision.is_err());

    clear_prefix(&prefix)?;
    Ok(())
}
