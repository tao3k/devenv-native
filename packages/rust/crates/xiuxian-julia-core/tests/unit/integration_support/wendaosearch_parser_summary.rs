use super::{
    parser_summary_ready_retry_delay_millis, parser_summary_ready_route_retries,
    parser_summary_ready_timeout_secs,
};
use serial_test::serial;

#[test]
#[serial]
fn parser_summary_ready_timeout_secs_uses_default() {
    super::set_parser_summary_env_for_tests(
        "WENDAOSEARCH_PARSER_SUMMARY_READY_TIMEOUT_SECS",
        Some("0"),
    );
    assert_eq!(parser_summary_ready_timeout_secs(), 4);
}

#[test]
#[serial]
fn parser_summary_ready_route_retries_uses_default() {
    super::set_parser_summary_env_for_tests(
        "WENDAOSEARCH_PARSER_SUMMARY_READY_ROUTE_RETRIES",
        Some("0"),
    );
    assert_eq!(parser_summary_ready_route_retries(), 6);
}

#[test]
#[serial]
fn parser_summary_ready_retry_delay_millis_uses_default() {
    super::set_parser_summary_env_for_tests(
        "WENDAOSEARCH_PARSER_SUMMARY_READY_RETRY_DELAY_MILLIS",
        Some("0"),
    );
    assert_eq!(parser_summary_ready_retry_delay_millis(), 250);
}
