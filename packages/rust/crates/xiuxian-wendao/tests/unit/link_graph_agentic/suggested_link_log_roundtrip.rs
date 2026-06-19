use super::{TEST_VALKEY_URL, clear_prefix, unique_prefix};
use xiuxian_wendao::{
    LinkGraphSuggestedLinkRequest, LinkGraphSuggestedLinkState,
    valkey_suggested_link_log_with_valkey, valkey_suggested_link_recent_with_valkey,
};

#[test]
fn test_suggested_link_log_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let prefix = unique_prefix();
    if clear_prefix(&prefix).is_err() {
        return Ok(());
    }

    let entry = valkey_suggested_link_log_with_valkey(
        &LinkGraphSuggestedLinkRequest {
            source_id: "docs/a.md".to_string(),
            target_id: "docs/b.md".to_string(),
            relation: "implements".to_string(),
            confidence: 0.83,
            evidence: "bridge signal from architecture section".to_string(),
            agent_id: "wendao-agentic-architect".to_string(),
            created_at_unix: Some(1_700_000_000.0),
        },
        TEST_VALKEY_URL,
        Some(&prefix),
        Some(10),
        Some(60),
    )
    .map_err(|err| err.clone())?;
    assert_eq!(
        entry.promotion_state,
        LinkGraphSuggestedLinkState::Provisional
    );
    assert!(!entry.suggestion_id.trim().is_empty());
    assert_eq!(entry.source_id, "docs/a.md");
    assert!((entry.updated_at_unix - entry.created_at_unix).abs() < 1e-9);

    let rows = valkey_suggested_link_recent_with_valkey(10, TEST_VALKEY_URL, Some(&prefix))
        .map_err(|err| err.clone())?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0], entry);

    clear_prefix(&prefix)?;
    Ok(())
}
