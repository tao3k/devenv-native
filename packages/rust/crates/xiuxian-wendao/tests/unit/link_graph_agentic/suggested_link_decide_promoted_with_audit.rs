#![allow(
    missing_docs,
    clippy::doc_markdown,
    clippy::implicit_clone,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::manual_string_new,
    clippy::needless_raw_string_hashes,
    clippy::format_push_string,
    clippy::unnecessary_to_owned,
    clippy::too_many_lines
)]
use super::{TEST_VALKEY_URL, clear_prefix, unique_prefix};
use xiuxian_wendao::{
    LinkGraphSuggestedLinkDecisionRequest, LinkGraphSuggestedLinkRequest,
    LinkGraphSuggestedLinkState, valkey_suggested_link_decide_with_valkey,
    valkey_suggested_link_decisions_recent_with_valkey, valkey_suggested_link_log_with_valkey,
    valkey_suggested_link_recent_latest_with_valkey,
};

#[test]
fn test_suggested_link_decide_promoted_with_audit() -> Result<(), Box<dyn std::error::Error>> {
    let prefix = unique_prefix();
    if clear_prefix(&prefix).is_err() {
        return Ok(());
    }

    let entry = valkey_suggested_link_log_with_valkey(
        &LinkGraphSuggestedLinkRequest {
            source_id: "docs/a.md".to_string(),
            target_id: "docs/b.md".to_string(),
            relation: "implements".to_string(),
            confidence: 0.9,
            evidence: "cross-reference".to_string(),
            agent_id: "qianhuan-architect".to_string(),
            created_at_unix: Some(1_700_000_100.0),
        },
        TEST_VALKEY_URL,
        Some(&prefix),
        Some(20),
        None,
    )
    .map_err(|err| err.to_string())?;

    let result = valkey_suggested_link_decide_with_valkey(
        &LinkGraphSuggestedLinkDecisionRequest {
            suggestion_id: entry.suggestion_id.clone(),
            target_state: LinkGraphSuggestedLinkState::Promoted,
            decided_by: "omega-gate".to_string(),
            reason: "passed gate checks".to_string(),
            decided_at_unix: Some(1_700_000_120.0),
        },
        TEST_VALKEY_URL,
        Some(&prefix),
        Some(20),
        None,
    )
    .map_err(|err| err.to_string())?;

    assert_eq!(
        result.suggestion.promotion_state,
        LinkGraphSuggestedLinkState::Promoted
    );
    assert_eq!(result.suggestion.decision_by.as_deref(), Some("omega-gate"));
    assert_eq!(
        result.suggestion.decision_reason.as_deref(),
        Some("passed gate checks")
    );
    assert_eq!(result.suggestion.updated_at_unix, 1_700_000_120.0);
    assert_eq!(
        result.decision.previous_state,
        LinkGraphSuggestedLinkState::Provisional
    );
    assert_eq!(
        result.decision.target_state,
        LinkGraphSuggestedLinkState::Promoted
    );

    let latest = valkey_suggested_link_recent_latest_with_valkey(
        10,
        TEST_VALKEY_URL,
        Some(&prefix),
        Some(LinkGraphSuggestedLinkState::Promoted),
        Some(50),
    )
    .map_err(|err| err.to_string())?;
    assert_eq!(latest.len(), 1);
    assert_eq!(latest[0].suggestion_id, entry.suggestion_id);
    assert_eq!(
        latest[0].promotion_state,
        LinkGraphSuggestedLinkState::Promoted
    );

    let decisions =
        valkey_suggested_link_decisions_recent_with_valkey(10, TEST_VALKEY_URL, Some(&prefix))
            .map_err(|err| err.to_string())?;
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].suggestion_id, entry.suggestion_id);
    assert_eq!(
        decisions[0].target_state,
        LinkGraphSuggestedLinkState::Promoted
    );

    clear_prefix(&prefix)?;
    Ok(())
}
