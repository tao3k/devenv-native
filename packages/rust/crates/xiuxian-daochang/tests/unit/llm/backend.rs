//! Test coverage for xiuxian-daochang behavior.

use serde_json::json;
use xiuxian_daochang::test_support::{
    ChatCompletionRequest, LlmBackendMode, build_responses_payload_from_chat_completion_request,
    extract_api_base_from_inference_url, is_openai_like_stream_required_error, parse_backend_mode,
    parse_responses_stream_tool_names, parse_tools_json, should_use_openai_like_for_base,
};

#[test]
fn parse_backend_mode_defaults_to_litellm_rs() {
    assert_eq!(parse_backend_mode(None), LlmBackendMode::LiteLlmRs);
    assert_eq!(parse_backend_mode(Some("")), LlmBackendMode::LiteLlmRs);
}

#[test]
fn parse_backend_mode_accepts_litellm_rs_aliases() {
    assert_eq!(
        parse_backend_mode(Some("litellm_rs")),
        LlmBackendMode::LiteLlmRs
    );
    assert_eq!(
        parse_backend_mode(Some("litellm-rs")),
        LlmBackendMode::LiteLlmRs
    );
}

#[test]
fn parse_backend_mode_invalid_value_falls_back_to_litellm_rs() {
    assert_eq!(
        parse_backend_mode(Some("unsupported-backend")),
        LlmBackendMode::LiteLlmRs
    );
}

#[test]
fn extract_api_base_from_inference_url_strips_completion_suffix() {
    let base = extract_api_base_from_inference_url("http://127.0.0.1:4000/v1/chat/completions");
    assert_eq!(base, "http://127.0.0.1:4000/v1");
}

#[test]
fn extract_api_base_from_inference_url_strips_anthropic_messages_suffix() {
    let base = extract_api_base_from_inference_url("https://aiproxy.xin/api/v1/messages");
    assert_eq!(base, "https://aiproxy.xin/api");
}

#[test]
fn openai_base_selection_uses_openai_like_for_custom_gateway() {
    assert!(!should_use_openai_like_for_base(
        "https://api.openai.com/v1"
    ));
    assert!(should_use_openai_like_for_base(
        "https://aiproxy.xin/openai"
    ));
    assert!(should_use_openai_like_for_base(
        "https://aiproxy.xin/openai/v1"
    ));
}

#[test]
fn detects_openai_like_stream_required_error_shape() {
    assert!(is_openai_like_stream_required_error(
        r#"API error for openai_like (status 400): {"detail":"Stream must be set to true"}"#,
    ));
    assert!(!is_openai_like_stream_required_error(
        r#"API error for openai_like (status 400): {"detail":"invalid request"}"#,
    ));
}

#[test]
fn parse_tools_json_keeps_name_description_and_schema() {
    let tools = parse_tools_json(Some(vec![serde_json::json!({
        "name": "crawl4ai.crawl_url",
        "description": "crawl web page",
        "input_schema": {"type":"object","properties":{"url":{"type":"string"}}}
    })]));
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "crawl4ai.crawl_url");
    assert_eq!(tools[0].description.as_deref(), Some("crawl web page"));
    assert!(tools[0].parameters.is_some());
}

#[test]
fn responses_payload_uses_input_and_stream_without_messages() {
    let payload = build_responses_payload_from_chat_completion_request(ChatCompletionRequest {
        model: "gpt-5-codex".to_string(),
        messages: vec![xiuxian_daochang::ChatMessage {
            role: "user".to_string(),
            content: Some("ping".to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }],
        max_tokens: Some(64),
        tools: None,
        tool_choice: None,
    })
    .unwrap_or_else(|error| panic!("responses payload should build: {error}"));

    assert_eq!(payload.get("stream"), Some(&serde_json::Value::Bool(true)));
    assert_eq!(
        payload.get("max_output_tokens"),
        Some(&serde_json::Value::from(64u32))
    );
    assert!(payload.get("messages").is_none());

    let input = payload
        .get("input")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("responses payload should contain input array"));
    assert_eq!(input.len(), 1);
    assert_eq!(
        input[0].get("role").and_then(serde_json::Value::as_str),
        Some("user")
    );
    assert_eq!(
        input[0].get("content"),
        Some(&serde_json::Value::String("ping".to_string()))
    );
}

#[test]
fn responses_payload_sanitizes_invalid_tool_names() {
    let payload = build_responses_payload_from_chat_completion_request(ChatCompletionRequest {
        model: "gpt-5-codex".to_string(),
        messages: vec![xiuxian_daochang::ChatMessage {
            role: "user".to_string(),
            content: Some("run tool".to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }],
        max_tokens: Some(64),
        tools: Some(vec![
            serde_json::json!({
                "name": "crawl4ai.crawl_url",
                "description": "crawl web page",
                "input_schema": {"type":"object","properties":{"url":{"type":"string"}}}
            }),
            serde_json::json!({
                "name": "crawl4ai_crawl_url",
                "description": "another tool with colliding sanitized name",
                "input_schema": {"type":"object","properties":{"url":{"type":"string"}}}
            }),
        ]),
        tool_choice: None,
    })
    .unwrap_or_else(|error| panic!("responses payload should build: {error}"));

    let tools = payload
        .get("tools")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("responses payload should contain tools array"));
    assert_eq!(
        tools[0].get("name").and_then(serde_json::Value::as_str),
        Some("crawl4ai_crawl_url")
    );
    assert_eq!(
        tools[1].get("name").and_then(serde_json::Value::as_str),
        Some("crawl4ai_crawl_url_2")
    );
}

#[test]
fn responses_stream_remaps_sanitized_tool_name_back_to_original() {
    let raw = r#"data: {"type":"response.output_item.done","item":{"type":"function_call","id":"call_1","call_id":"call_1","name":"crawl4ai_crawl_url","arguments":"{\"url\":\"https://example.com\"}"}}
data: [DONE]"#;

    let tool_names = parse_responses_stream_tool_names(
        raw,
        &[(
            "crawl4ai_crawl_url".to_string(),
            "crawl4ai.crawl_url".to_string(),
        )],
    )
    .unwrap_or_else(|error| panic!("responses stream should parse: {error}"));

    assert_eq!(tool_names, vec!["crawl4ai.crawl_url".to_string()]);
}

#[test]
fn responses_payload_preserves_tool_call_chain_from_daochang_messages() {
    let payload = build_responses_payload_from_chat_completion_request(ChatCompletionRequest {
        model: "gpt-5-codex".to_string(),
        messages: vec![
            xiuxian_daochang::ChatMessage {
                role: "user".to_string(),
                content: Some("show me the agenda".to_string()),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
            xiuxian_daochang::ChatMessage {
                role: "assistant".to_string(),
                content: Some("Checking the agenda first.".to_string()),
                tool_calls: Some(vec![xiuxian_daochang::ToolCallOut {
                    id: "call_123".to_string(),
                    typ: "function".to_string(),
                    function: xiuxian_daochang::FunctionCall {
                        name: "agenda.view".to_string(),
                        arguments: "{}".to_string(),
                    },
                }]),
                tool_call_id: None,
                name: None,
            },
            xiuxian_daochang::ChatMessage {
                role: "tool".to_string(),
                content: Some(r#"{"ok":true}"#.to_string()),
                tool_calls: None,
                tool_call_id: Some("call_123".to_string()),
                name: Some("agenda.view".to_string()),
            },
        ],
        max_tokens: Some(64),
        tools: Some(vec![json!({
            "name": "agenda.view",
            "description": "view agenda",
            "input_schema": {"type": "object", "properties": {}}
        })]),
        tool_choice: None,
    })
    .unwrap_or_else(|error| panic!("responses payload should build: {error}"));

    let input = payload
        .get("input")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("responses payload should contain input array"));
    let function_call = input
        .iter()
        .find(|item| item.get("type").and_then(serde_json::Value::as_str) == Some("function_call"))
        .unwrap_or_else(|| panic!("assistant function call should be present"));
    let function_output = input
        .iter()
        .find(|item| {
            item.get("type").and_then(serde_json::Value::as_str) == Some("function_call_output")
        })
        .unwrap_or_else(|| panic!("tool output should be present"));

    assert_eq!(function_call.get("call_id"), Some(&json!("call_123")));
    assert_eq!(function_call.get("name"), Some(&json!("agenda_view")));
    assert_eq!(function_output.get("call_id"), Some(&json!("call_123")));
    assert_eq!(
        function_output.get("output"),
        Some(&json!(r#"{"ok":true}"#))
    );
}
