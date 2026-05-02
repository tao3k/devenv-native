use super::{TEST_VALKEY_URL, clear_prefix, unique_prefix};
use xiuxian_wendao::{
    LinkGraphSuggestedLinkRequest, valkey_suggested_link_log_with_valkey,
    valkey_suggested_link_recent_with_valkey,
};

#[test]
fn test_suggested_link_log_trims_stream_by_max_entries() -> Result<(), Box<dyn std::error::Error>> {
    let prefix = unique_prefix();
    if clear_prefix(&prefix).is_err() {
        return Ok(());
    }

    for idx in 0..3 {
        let _ = valkey_suggested_link_log_with_valkey(
            &LinkGraphSuggestedLinkRequest {
                source_id: format!("docs/source-{idx}.md"),
                target_id: format!("docs/target-{idx}.md"),
                relation: "related_to".to_string(),
                confidence: 0.5,
                evidence: "test".to_string(),
                agent_id: "qianhuan-architect".to_string(),
                created_at_unix: Some(1_700_000_000.0 + f64::from(idx)),
            },
            TEST_VALKEY_URL,
            Some(&prefix),
            Some(2),
            None,
        )
        .map_err(|err| err.clone())?;
    }

    let rows = valkey_suggested_link_recent_with_valkey(10, TEST_VALKEY_URL, Some(&prefix))
        .map_err(|err| err.clone())?;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].source_id, "docs/source-2.md");
    assert_eq!(rows[1].source_id, "docs/source-1.md");

    clear_prefix(&prefix)?;
    Ok(())
}
