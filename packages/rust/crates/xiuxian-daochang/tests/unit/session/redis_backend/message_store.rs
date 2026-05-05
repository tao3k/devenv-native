use xiuxian_daochang::test_support::{decode_chat_message_payload, encode_chat_message_payload};
use xiuxian_daochang::{ChatMessage, FunctionCall, ToolCallOut};

#[test]
fn compact_chat_message_payload_roundtrip_preserves_tool_calls()
-> Result<(), Box<dyn std::error::Error>> {
    let message = ChatMessage {
        role: "assistant".to_string(),
        content: Some("tool execution completed".to_string()),
        tool_calls: Some(vec![ToolCallOut {
            id: "call-1".to_string(),
            typ: "function".to_string(),
            function: FunctionCall {
                name: "knowledge.search".to_string(),
                arguments: "{\"query\":\"graph memory\"}".to_string(),
            },
        }]),
        tool_call_id: Some("tool-call-result-1".to_string()),
        name: Some("knowledge.search".to_string()),
    };

    let encoded = encode_chat_message_payload(&message, None)?;
    assert!(!encoded.content_truncated);
    assert!(encoded.payload.contains("\"r\":\"assistant\""));
    assert!(encoded.payload.contains("\"tc\""));
    assert!(!encoded.payload.contains("\"role\""));

    let decoded = decode_chat_message_payload("session-1", &encoded.payload)?;
    assert_eq!(decoded.role, "assistant");
    assert_eq!(decoded.content.as_deref(), Some("tool execution completed"));
    assert_eq!(decoded.tool_call_id.as_deref(), Some("tool-call-result-1"));
    assert_eq!(decoded.name.as_deref(), Some("knowledge.search"));
    let tool_calls = decoded
        .tool_calls
        .ok_or_else(|| std::io::Error::other("decoded message should include tool calls"))?;
    assert_eq!(tool_calls.len(), 1);
    assert_eq!(tool_calls[0].id, "call-1");
    assert_eq!(tool_calls[0].typ, "function");
    assert_eq!(tool_calls[0].function.name, "knowledge.search");
    assert_eq!(
        tool_calls[0].function.arguments,
        "{\"query\":\"graph memory\"}"
    );
    Ok(())
}

#[test]
fn decode_chat_message_payload_accepts_legacy_role_content_shape()
-> Result<(), Box<dyn std::error::Error>> {
    let payload = r#"{"role":"user","content":"hello","name":"speaker"}"#;
    let decoded = decode_chat_message_payload("session-legacy", payload)?;
    assert_eq!(decoded.role, "user");
    assert_eq!(decoded.content.as_deref(), Some("hello"));
    assert_eq!(decoded.name.as_deref(), Some("speaker"));
    Ok(())
}

#[test]
fn decode_chat_message_payload_preserves_legacy_backup_metadata_payload()
-> Result<(), Box<dyn std::error::Error>> {
    let session_id = "__session_context_backup_meta__:session-42";
    let payload = r#"{"messages":4,"summary_segments":1,"saved_at_unix_ms":1771623456789}"#;
    let decoded = decode_chat_message_payload(session_id, payload)?;
    assert_eq!(decoded.role, "system");
    assert_eq!(decoded.content.as_deref(), Some(payload));
    assert!(decoded.tool_calls.is_none());
    Ok(())
}

#[test]
fn encode_chat_message_payload_truncates_content_when_limit_configured()
-> Result<(), Box<dyn std::error::Error>> {
    let message = ChatMessage {
        role: "user".to_string(),
        content: Some("abcdefghij".to_string()),
        tool_calls: None,
        tool_call_id: None,
        name: None,
    };

    let encoded = encode_chat_message_payload(&message, Some(4))?;
    assert!(encoded.content_truncated);

    let decoded = decode_chat_message_payload("session-truncate", &encoded.payload)?;
    assert_eq!(decoded.content.as_deref(), Some("abcd"));
    Ok(())
}
