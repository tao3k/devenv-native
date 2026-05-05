use super::support::{
    ChatMessage, Client, LiteChatRequest, MessageContent, MessageRole, MockResponse, Ordering,
    Result, StatusCode, anyhow, execute_openai_responses_request,
    spawn_mock_responses_sequence_server,
};

#[tokio::test]
async fn execute_openai_responses_request_rejects_duplicate_tool_outputs_before_send() -> Result<()>
{
    let (endpoint, requests_seen) = spawn_mock_responses_sequence_server(vec![MockResponse {
        status: StatusCode::OK,
        content_type: "text/event-stream",
        body: r#"data: {"type":"response.output_item.done","item":{"type":"message","content":[{"type":"output_text","text":"pong"}]}}
data: [DONE]"#,
    }])
    .await?;

    let request = LiteChatRequest {
        model: "gpt-5-codex".to_string(),
        messages: vec![
            ChatMessage {
                role: MessageRole::Assistant,
                tool_calls: Some(vec![litellm_rs::core::types::tools::ToolCall {
                    id: "call_dup".to_string(),
                    tool_type: "function".to_string(),
                    function: litellm_rs::core::types::tools::FunctionCall {
                        name: "qianhuan.reload".to_string(),
                        arguments: r#"{"scope":"all"}"#.to_string(),
                    },
                }]),
                ..Default::default()
            },
            ChatMessage {
                role: MessageRole::Tool,
                content: Some(MessageContent::Text("first tool output".to_string())),
                tool_call_id: Some("call_dup".to_string()),
                ..Default::default()
            },
            ChatMessage {
                role: MessageRole::Tool,
                content: Some(MessageContent::Text("duplicate tool output".to_string())),
                tool_call_id: Some("call_dup".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let err =
        execute_openai_responses_request(&Client::new(), &endpoint, Some("test-key"), &request)
            .await
            .err()
            .ok_or_else(|| anyhow!("duplicate tool output should fail locally"))?;

    let rendered = err.to_string();
    if !rendered.contains("without an available preceding function_call") {
        return Err(anyhow!("unexpected error message: {rendered}"));
    }
    assert_eq!(requests_seen.load(Ordering::SeqCst), 0);
    Ok(())
}

#[tokio::test]
async fn execute_openai_responses_request_rejects_orphan_tool_outputs_before_send() -> Result<()> {
    let (endpoint, requests_seen) = spawn_mock_responses_sequence_server(vec![MockResponse {
        status: StatusCode::OK,
        content_type: "text/event-stream",
        body: r#"data: {"type":"response.output_item.done","item":{"type":"message","content":[{"type":"output_text","text":"pong"}]}}
data: [DONE]"#,
    }])
    .await?;

    let request = LiteChatRequest {
        model: "gpt-5-codex".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::Tool,
            content: Some(MessageContent::Text("orphan tool output".to_string())),
            tool_call_id: Some("call_orphan".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };

    let err =
        execute_openai_responses_request(&Client::new(), &endpoint, Some("test-key"), &request)
            .await
            .err()
            .ok_or_else(|| anyhow!("orphan tool output should fail locally"))?;

    let rendered = err.to_string();
    if !rendered.contains("function_call_output items without an available preceding function_call")
    {
        return Err(anyhow!("unexpected error message: {rendered}"));
    }
    assert_eq!(requests_seen.load(Ordering::SeqCst), 0);
    Ok(())
}
