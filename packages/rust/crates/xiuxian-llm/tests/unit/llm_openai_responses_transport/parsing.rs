use super::support::*;

#[tokio::test]
async fn execute_openai_responses_request_parses_text_and_tool_calls() -> Result<()> {
    let endpoint = spawn_mock_responses_server(MockResponse {
        status: StatusCode::OK,
        content_type: "text/event-stream",
        body: r#"data: {"type":"response.output_item.done","item":{"type":"message","content":[{"type":"output_text","text":"pong"}]}}
data: {"type":"response.output_item.done","item":{"type":"function_call","id":"call_1","call_id":"call_1","name":"qianhuan_reload","arguments":"{\"scope\":\"all\"}"}}
data: [DONE]"#,
    })
    .await?;

    let parsed = execute_openai_responses_request(
        &Client::new(),
        &endpoint,
        Some("test-key"),
        &request_with_tool_alias(),
    )
    .await?;

    assert_eq!(parsed.content.as_deref(), Some("pong"));
    assert_eq!(parsed.tool_calls.len(), 1);
    assert_eq!(parsed.tool_calls[0].function.name, "qianhuan.reload");
    assert_eq!(
        parsed.tool_calls[0].function.arguments,
        r#"{"scope":"all"}"#
    );
    Ok(())
}

#[tokio::test]
async fn execute_openai_responses_request_sends_developer_role_in_payload() -> Result<()> {
    let (endpoint, requests) = spawn_mock_captured_responses_server(MockResponse {
        status: StatusCode::OK,
        content_type: "text/event-stream",
        body: r#"data: {"type":"response.output_item.done","item":{"type":"message","content":[{"type":"output_text","text":"pong"}]}}
data: [DONE]"#,
    })
    .await?;

    let request = LiteChatRequest {
        model: "gpt-5-codex".to_string(),
        messages: vec![
            ChatMessage {
                role: MessageRole::Developer,
                content: Some(MessageContent::Text(
                    "Prefer terse implementation-first answers.".to_string(),
                )),
                ..Default::default()
            },
            ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("hello".to_string())),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let parsed = execute_openai_responses_request(
        &Client::new(),
        &endpoint,
        Some("test-key"),
        &request,
    )
    .await?;

    assert_eq!(parsed.content.as_deref(), Some("pong"));
    let captured = match requests.lock() {
        Ok(captured) => captured.clone(),
        Err(error) => panic!("capture lock should not be poisoned: {error}"),
    };
    let payload = captured
        .first()
        .ok_or_else(|| anyhow!("expected a captured request payload"))?;
    let input = payload["input"]
        .as_array()
        .ok_or_else(|| anyhow!("captured payload should include input array: {payload}"))?;

    assert_eq!(input[0]["role"], json!("developer"));
    assert_eq!(
        input[0]["content"],
        json!("Prefer terse implementation-first answers.")
    );
    assert_eq!(input[1]["role"], json!("user"));
    Ok(())
}
