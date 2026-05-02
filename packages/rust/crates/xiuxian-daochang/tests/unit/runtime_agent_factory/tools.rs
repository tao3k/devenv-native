use crate::RuntimeSettings;

use super::resolve_runtime_tool_options;

#[test]
fn resolve_runtime_tool_options_uses_expected_defaults() {
    let options = resolve_runtime_tool_options(&RuntimeSettings::default());
    assert_eq!(options.pool_size, 8);
    assert_eq!(options.handshake_timeout_secs, 10);
    assert_eq!(options.connect_retries, 2);
    assert!(!options.strict_startup);
    assert_eq!(options.connect_retry_backoff_ms, 500);
    assert_eq!(options.tool_timeout_secs, 30);
    assert_eq!(options.list_tools_cache_ttl_ms, 5_000);
}
