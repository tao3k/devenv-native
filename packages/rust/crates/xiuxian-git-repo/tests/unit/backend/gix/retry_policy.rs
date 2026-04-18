use super::{is_retryable_remote_error_message, retry_delay_for_attempt};

#[test]
fn retryable_remote_error_message_matches_transient_transport_failures() {
    assert!(is_retryable_remote_error_message(
        "failed to connect to github.com: Can't assign requested address; class=Os (2)"
    ));
    assert!(is_retryable_remote_error_message(
        "connection reset by peer while fetching packfile"
    ));
    assert!(is_retryable_remote_error_message(
        "operation timed out after 30 seconds"
    ));
}

#[test]
fn retryable_remote_error_message_rejects_non_transient_failures() {
    assert!(!is_retryable_remote_error_message(
        "authentication required but no callback set"
    ));
    assert!(!is_retryable_remote_error_message("reference not found"));
}

#[test]
fn retry_delay_for_attempt_caps_backoff_growth() {
    assert_eq!(retry_delay_for_attempt(1).as_millis(), 250);
    assert_eq!(retry_delay_for_attempt(2).as_millis(), 500);
    assert_eq!(retry_delay_for_attempt(3).as_millis(), 1000);
    assert_eq!(retry_delay_for_attempt(9).as_millis(), 1000);
}
