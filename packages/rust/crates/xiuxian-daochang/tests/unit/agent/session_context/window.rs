use anyhow::Result;

use xiuxian_daochang::test_support::{
    bounded_recent_messages, bounded_recent_summary_segments, build_session_context_test_agent,
    enforce_session_reset_policy, now_unix_ms, session_messages, set_session_last_activity,
    set_session_reset_idle_timeout_ms,
};

const SESSION_RESET_NOTICE_NAME: &str = "session_reset_notice";
const IDLE_RESET_NOTICE_TEXT: &str = "Previous session expired due to inactivity.";

#[tokio::test]
async fn enforce_session_reset_policy_resets_stale_unbounded_session_and_injects_notice()
-> Result<()> {
    let mut agent = build_session_context_test_agent(None).await?;
    set_session_reset_idle_timeout_ms(&mut agent, Some(1));
    let session_id = "session-reset-unbounded";

    agent
        .append_turn_for_session(session_id, "u1", "a1")
        .await?;
    let stale_ms = now_unix_ms().saturating_sub(10);
    set_session_last_activity(&agent, session_id, stale_ms).await;

    enforce_session_reset_policy(&agent, session_id).await?;

    let messages = session_messages(&agent, session_id).await?;
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].name.as_deref(), Some(SESSION_RESET_NOTICE_NAME));
    assert_eq!(messages[0].content.as_deref(), Some(IDLE_RESET_NOTICE_TEXT));

    let backup = agent.peek_context_window_backup(session_id).await?;
    assert!(
        backup.is_some(),
        "stale reset should preserve backup snapshot"
    );

    Ok(())
}

#[tokio::test]
async fn enforce_session_reset_policy_resets_stale_bounded_session_and_injects_summary_notice()
-> Result<()> {
    let mut agent = build_session_context_test_agent(Some(8)).await?;
    set_session_reset_idle_timeout_ms(&mut agent, Some(1));
    let session_id = "session-reset-bounded";

    agent
        .append_turn_for_session(session_id, "u1", "a1")
        .await?;
    agent
        .append_turn_for_session(session_id, "u2", "a2")
        .await?;
    let stale_ms = now_unix_ms().saturating_sub(10);
    set_session_last_activity(&agent, session_id, stale_ms).await;

    enforce_session_reset_policy(&agent, session_id).await?;

    let recent_messages = bounded_recent_messages(&agent, session_id, 16).await?;
    assert!(
        recent_messages.is_empty(),
        "stale reset should clear active window"
    );

    let summary_segments = bounded_recent_summary_segments(&agent, session_id, 8).await?;
    assert_eq!(summary_segments.len(), 1);
    assert_eq!(summary_segments[0].summary, IDLE_RESET_NOTICE_TEXT);

    let backup = agent.peek_context_window_backup(session_id).await?;
    assert!(
        backup.is_some(),
        "stale reset should preserve backup snapshot"
    );

    Ok(())
}
