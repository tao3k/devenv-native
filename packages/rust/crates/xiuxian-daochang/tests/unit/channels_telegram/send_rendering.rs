use super::support::{
    Channel, Result, TelegramChannel, spawn_mock_telegram_api, spawn_mock_telegram_api_level_error,
};

#[tokio::test]
async fn telegram_send_uses_markdown_v2_parse_mode_with_rendering() -> Result<()> {
    let Some((api_base, state, handle)) = spawn_mock_telegram_api(None).await? else {
        return Ok(());
    };
    let channel = TelegramChannel::new_with_base_url(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        api_base,
    );

    channel
        .send(
            "**bold** [link](https://example.com) `code` <raw>",
            "123456",
        )
        .await?;

    let requests = state.requests.lock().await;
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    assert_eq!(
        request
            .get("parse_mode")
            .and_then(serde_json::Value::as_str),
        Some("MarkdownV2")
    );
    let rendered_text = request
        .get("text")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    assert!(rendered_text.contains("*bold*"));
    assert!(rendered_text.contains("[link](https://example.com)"));
    assert!(rendered_text.contains("`code`"));
    assert!(
        rendered_text.contains("<raw\\>") || rendered_text.contains("\\<raw\\>"),
        "expected escaped raw marker in MarkdownV2 payload, got: {rendered_text}"
    );

    handle.abort();
    Ok(())
}

#[tokio::test]
async fn telegram_send_includes_message_thread_id_when_recipient_has_topic_suffix() -> Result<()> {
    let Some((api_base, state, handle)) = spawn_mock_telegram_api(None).await? else {
        return Ok(());
    };
    let channel = TelegramChannel::new_with_base_url(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        api_base,
    );

    channel.send("topic hello", "123456:42").await?;

    let requests = state.requests.lock().await;
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    assert_eq!(
        request.get("chat_id").and_then(serde_json::Value::as_str),
        Some("123456")
    );
    assert_eq!(
        request
            .get("message_thread_id")
            .and_then(serde_json::Value::as_str),
        Some("42")
    );

    handle.abort();
    Ok(())
}

#[tokio::test]
async fn telegram_send_falls_back_to_html_after_markdown_parse_error() -> Result<()> {
    let Some((api_base, state, handle)) =
        spawn_mock_telegram_api(Some("Bad Request: can't parse entities")).await?
    else {
        return Ok(());
    };
    let channel = TelegramChannel::new_with_base_url(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        api_base,
    );

    channel.send("fallback check", "123456").await?;

    let requests = state.requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert_eq!(
        requests[0]
            .get("parse_mode")
            .and_then(serde_json::Value::as_str),
        Some("MarkdownV2")
    );
    assert_eq!(
        requests[1]
            .get("parse_mode")
            .and_then(serde_json::Value::as_str),
        Some("HTML")
    );
    assert_eq!(
        requests[1].get("text").and_then(serde_json::Value::as_str),
        Some("fallback check")
    );

    handle.abort();
    Ok(())
}

#[tokio::test]
async fn telegram_send_chunk_markers_are_plain_text() -> Result<()> {
    let Some((api_base, state, handle)) = spawn_mock_telegram_api(None).await? else {
        return Ok(());
    };
    let channel = TelegramChannel::new_with_base_url(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        api_base,
    );

    let message = "a".repeat(4300);
    channel.send(&message, "123456").await?;

    let requests = state.requests.lock().await;
    assert!(requests.len() >= 2, "long messages should be split");
    let first = requests[0]
        .get("text")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let second = requests[1]
        .get("text")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    assert!(first.contains("\\(continues\\.\\.\\.\\)"));
    assert!(second.contains("\\(continued\\)"));
    assert!(!first.contains("_(continues...)_"));
    assert!(!second.contains("_(continued)_"));

    handle.abort();
    Ok(())
}

#[tokio::test]
async fn telegram_send_falls_back_to_html_on_generic_markdown_bad_request() -> Result<()> {
    let Some((api_base, state, handle)) =
        spawn_mock_telegram_api(Some("Bad Request: markdown rejected")).await?
    else {
        return Ok(());
    };
    let channel = TelegramChannel::new_with_base_url(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        api_base,
    );

    channel.send("fallback check", "123456").await?;

    let requests = state.requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert_eq!(
        requests[0]
            .get("parse_mode")
            .and_then(serde_json::Value::as_str),
        Some("MarkdownV2")
    );
    assert_eq!(
        requests[1]
            .get("parse_mode")
            .and_then(serde_json::Value::as_str),
        Some("HTML")
    );
    assert_eq!(
        requests[1].get("text").and_then(serde_json::Value::as_str),
        Some("fallback check")
    );

    handle.abort();
    Ok(())
}

#[tokio::test]
async fn telegram_send_falls_back_to_html_on_markdown_api_error_with_http_200() -> Result<()> {
    let Some((api_base, state, handle)) =
        spawn_mock_telegram_api_level_error(Some("Bad Request: markdown rejected")).await?
    else {
        return Ok(());
    };
    let channel = TelegramChannel::new_with_base_url(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        api_base,
    );

    channel.send("fallback check", "123456").await?;

    let requests = state.requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert_eq!(
        requests[0]
            .get("parse_mode")
            .and_then(serde_json::Value::as_str),
        Some("MarkdownV2")
    );
    assert_eq!(
        requests[1]
            .get("parse_mode")
            .and_then(serde_json::Value::as_str),
        Some("HTML")
    );
    assert_eq!(
        requests[1].get("text").and_then(serde_json::Value::as_str),
        Some("fallback check")
    );

    handle.abort();
    Ok(())
}
