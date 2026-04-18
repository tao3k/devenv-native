use anyhow::Result;

use super::support::{parse_array_reply, parse_map_reply};
use redis::Value;
use xiuxian_daochang::test_support::{
    StreamReadErrorKind, build_consumer_name, classify_stream_read_error, compute_retry_backoff_ms,
    is_idle_poll_timeout_error, parse_xreadgroup_reply, should_surface_repeated_failure,
    stream_consumer_response_timeout, summarize_redis_error,
};

#[test]
fn parse_xreadgroup_reply_nil_returns_empty() -> Result<()> {
    let events = parse_xreadgroup_reply(Value::Nil)?;
    assert!(events.is_empty());
    Ok(())
}

#[test]
fn parse_xreadgroup_reply_array_extracts_events() -> Result<()> {
    let events = parse_array_reply()?;
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].id, "1740000000000-0");
    assert_eq!(
        events[0].fields.get("kind").map(String::as_str),
        Some("turn_stored")
    );
    assert_eq!(
        events[0].fields.get("session_id").map(String::as_str),
        Some("telegram:1:1")
    );
    assert_eq!(events[1].id, "1740000000001-0");
    assert_eq!(
        events[1].fields.get("kind").map(String::as_str),
        Some("consolidation_stored")
    );
    Ok(())
}

#[test]
fn parse_xreadgroup_reply_map_extracts_events() -> Result<()> {
    let events = parse_map_reply()?;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].id, "1740000001000-0");
    assert_eq!(
        events[0].fields.get("kind").map(String::as_str),
        Some("recall_snapshot_updated")
    );
    assert_eq!(
        events[0].fields.get("session_id").map(String::as_str),
        Some("telegram:9:9")
    );
    Ok(())
}

#[test]
fn build_consumer_name_keeps_prefix() {
    let name = build_consumer_name("agent-test");
    assert!(name.starts_with("agent-test-"));
}

#[test]
fn classify_stream_read_error_detects_missing_group() {
    let error = anyhow::anyhow!("xreadgroup failed for stream_id=>: NOGROUP No such key");
    let kind = classify_stream_read_error(&error);
    assert_eq!(kind, StreamReadErrorKind::MissingConsumerGroup);
}

#[test]
fn classify_stream_read_error_detects_transport() {
    let error =
        anyhow::anyhow!("xreadgroup failed for stream_id=>: Connection reset by peer while read");
    let kind = classify_stream_read_error(&error);
    assert_eq!(kind, StreamReadErrorKind::Transport);
}

#[test]
fn classify_stream_read_error_falls_back_to_other() {
    let error = anyhow::anyhow!("xreadgroup failed for stream_id=>: some unknown parser issue");
    let kind = classify_stream_read_error(&error);
    assert_eq!(kind, StreamReadErrorKind::Other);
}

#[test]
fn classify_stream_read_error_uses_error_chain() {
    let error = anyhow::anyhow!("timed out while waiting for redis reply")
        .context("xreadgroup failed for stream_id=>");
    let kind = classify_stream_read_error(&error);
    assert_eq!(kind, StreamReadErrorKind::Transport);
}

#[test]
fn idle_poll_timeout_error_detects_timeout_like_io_error_text() {
    let error = redis::RedisError::from((redis::ErrorKind::Io, "operation timed out"));
    assert!(is_idle_poll_timeout_error(&error));
}

#[test]
fn idle_poll_timeout_error_ignores_non_timeout_io_errors() {
    let error = redis::RedisError::from((redis::ErrorKind::Io, "connection reset by peer"));
    assert!(!is_idle_poll_timeout_error(&error));
}

#[test]
fn summarize_redis_error_includes_kind_and_category() {
    let error = redis::RedisError::from((redis::ErrorKind::Io, "operation timed out"));
    let summary = summarize_redis_error(&error);
    assert!(summary.contains("kind=Io"), "summary={summary}");
    assert!(summary.contains("category=I/O error"), "summary={summary}");
    assert!(summary.contains("timeout="), "summary={summary}");
}

#[test]
fn stream_consumer_response_timeout_exceeds_block_timeout() {
    let timeout = stream_consumer_response_timeout(1_000);
    assert_eq!(timeout.as_millis(), 1_500);
}

#[test]
fn compute_retry_backoff_ms_exponential_and_capped() {
    assert_eq!(compute_retry_backoff_ms(500, 1), 500);
    assert_eq!(compute_retry_backoff_ms(500, 2), 1_000);
    assert_eq!(compute_retry_backoff_ms(500, 3), 2_000);
    assert_eq!(compute_retry_backoff_ms(500, 20), 30_000);
}

#[test]
fn should_surface_repeated_failure_throttles_noise() {
    assert!(should_surface_repeated_failure(1));
    assert!(should_surface_repeated_failure(2));
    assert!(!should_surface_repeated_failure(3));
    assert!(should_surface_repeated_failure(4));
    assert!(!should_surface_repeated_failure(19));
    assert!(should_surface_repeated_failure(20));
}
