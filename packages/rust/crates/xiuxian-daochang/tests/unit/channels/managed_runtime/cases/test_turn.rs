use xiuxian_daochang::test_support::{build_session_id, classify_turn_error};

#[test]
fn managed_turn_build_session_id_joins_channel_and_session_key() {
    let session_id = build_session_id("telegram", "group:42:1001");
    assert_eq!(session_id, "telegram:group:42:1001");
}

#[test]
fn managed_turn_classify_error_detects_known_categories_case_insensitive() {
    let cases = [
        ("tool tools/list failed", "tool_runtime_tools_list"),
        ("invoke TOOLS/CALL timeout", "tool_runtime_tools_call"),
        (
            "transport send error: broken pipe",
            "tool_runtime_transport",
        ),
        (
            "tool handshake timeout while booting",
            "tool_runtime_connect",
        ),
        ("LLM provider failed", "llm"),
        ("unexpected parser issue", "unknown"),
    ];

    for (error, expected) in cases {
        assert_eq!(classify_turn_error(error), expected, "input={error}");
    }
}

#[test]
fn managed_turn_classify_error_prioritizes_tools_list_bucket() {
    let error = "tools/list returned error sending request";
    assert_eq!(classify_turn_error(error), "tool_runtime_tools_list");
}
