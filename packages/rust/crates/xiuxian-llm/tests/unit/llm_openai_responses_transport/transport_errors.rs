use super::support::*;

#[tokio::test]
async fn execute_openai_responses_request_retries_transient_503_and_succeeds() -> Result<()> {
    let (endpoint, requests_seen) = spawn_mock_responses_sequence_server(vec![
        MockResponse {
            status: StatusCode::SERVICE_UNAVAILABLE,
            content_type: "text/event-stream",
            body: "upstream connect error or disconnect/reset before headers. reset reason: connection termination",
        },
        MockResponse {
            status: StatusCode::OK,
            content_type: "text/event-stream",
            body: r#"data: {"type":"response.output_item.done","item":{"type":"message","content":[{"type":"output_text","text":"pong"}]}}
data: [DONE]"#,
        },
    ])
    .await?;

    let parsed = execute_openai_responses_request(
        &Client::new(),
        &endpoint,
        Some("test-key"),
        &request_with_tool_alias(),
    )
    .await?;

    assert_eq!(parsed.content.as_deref(), Some("pong"));
    assert_eq!(requests_seen.load(Ordering::SeqCst), 2);
    Ok(())
}

#[tokio::test]
async fn execute_openai_responses_request_surfaces_http_error_status() -> Result<()> {
    let endpoint = spawn_mock_responses_server(MockResponse {
        status: StatusCode::BAD_REQUEST,
        content_type: "application/json",
        body: r#"{"error":{"message":"invalid request"}}"#,
    })
    .await?;

    let err = execute_openai_responses_request(
        &Client::new(),
        &endpoint,
        Some("test-key"),
        &request_with_tool_alias(),
    )
    .await
    .err()
    .ok_or_else(|| anyhow!("400 status should fail"))?;
    let rendered = err.to_string();
    if !rendered.contains("status 400") {
        return Err(anyhow!("unexpected error message: {rendered}"));
    }
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn execute_openai_responses_request_fails_fast_when_headers_stall() -> Result<()> {
    let endpoint = spawn_mock_delayed_responses_server(
        MockResponse {
            status: StatusCode::OK,
            content_type: "text/event-stream",
            body: "data: [DONE]",
        },
        Duration::from_mins(1),
    )
    .await?;

    let request = request_with_tool_alias();
    let task = tokio::spawn(async move {
        execute_openai_responses_request(&Client::new(), &endpoint, Some("test-key"), &request)
            .await
    });

    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(31)).await;

    let result = task.await?;
    let err = result
        .err()
        .ok_or_else(|| anyhow!("stalled headers should fail"))?;
    let rendered = err.to_string();
    if !rendered.contains("response headers were not received within 10s") {
        return Err(anyhow!("unexpected timeout error message: {rendered}"));
    }
    Ok(())
}

#[test]
fn stream_required_detector_matches_expected_error_shape() {
    assert!(is_openai_like_stream_required_error_message(
        r#"API error for openai_like (status 400): {"detail":"Stream must be set to true"}"#,
    ));
    assert!(!is_openai_like_stream_required_error_message(
        r#"API error for openai_like (status 400): {"detail":"invalid request"}"#,
    ));
}
