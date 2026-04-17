#![allow(
    missing_docs,
    unused_imports,
    dead_code,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::field_reassign_with_default,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::map_unwrap_or,
    clippy::option_as_ref_deref,
    clippy::unreadable_literal,
    clippy::useless_conversion,
    clippy::match_wildcard_for_single_variants,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_raw_string_hashes,
    clippy::manual_async_fn,
    clippy::manual_let_else,
    clippy::manual_assert,
    clippy::manual_string_new,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::unnecessary_literal_bound,
    clippy::needless_pass_by_value,
    clippy::struct_field_names,
    clippy::single_match_else,
    clippy::similar_names,
    clippy::format_collect,
    clippy::async_yields_async,
    clippy::assigning_clones
)]

use anyhow::Result;
use xiuxian_daochang::{Channel, TelegramChannel};

use super::telegram_media_support::{
    spawn_mock_telegram_media_api, spawn_mock_telegram_media_api_with_group_failure,
};

#[tokio::test]
async fn telegram_media_markers_map_all_attachment_types() -> Result<()> {
    let Some((api_base, state, handle)) = spawn_mock_telegram_media_api().await? else {
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
            "[IMAGE:https://example.com/a.png]\
[DOCUMENT:https://example.com/a.pdf]\
[VIDEO:https://example.com/a.mp4]\
[AUDIO:https://example.com/a.mp3]\
[VOICE:https://example.com/a.ogg]",
            "123456",
        )
        .await?;

    let calls = state.calls.lock().await;
    let methods: Vec<&str> = calls.iter().map(|c| c.method.as_str()).collect();
    assert_eq!(methods, vec!["sendMediaGroup", "sendVoice"]);
    let media = calls[0]
        .payload
        .get("media")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert_eq!(media.len(), 4);
    let media_types: Vec<&str> = media
        .iter()
        .filter_map(|item| item.get("type").and_then(serde_json::Value::as_str))
        .collect();
    assert_eq!(media_types, vec!["photo", "document", "video", "audio"]);

    handle.abort();
    Ok(())
}

#[tokio::test]
async fn telegram_media_group_uses_send_media_group_for_multi_url_payload() -> Result<()> {
    let Some((api_base, state, handle)) = spawn_mock_telegram_media_api().await? else {
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
            "[IMAGE:https://example.com/a.png][DOCUMENT:https://example.com/b.pdf]",
            "123456",
        )
        .await?;

    let calls = state.calls.lock().await;
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].method, "sendMediaGroup");
    let media = calls[0]
        .payload
        .get("media")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert_eq!(media.len(), 2);
    assert_eq!(
        media[0].get("type").and_then(serde_json::Value::as_str),
        Some("photo")
    );
    assert_eq!(
        media[0].get("media").and_then(serde_json::Value::as_str),
        Some("https://example.com/a.png")
    );
    assert_eq!(
        media[1].get("type").and_then(serde_json::Value::as_str),
        Some("document")
    );
    assert_eq!(
        media[1].get("media").and_then(serde_json::Value::as_str),
        Some("https://example.com/b.pdf")
    );

    handle.abort();
    Ok(())
}

#[tokio::test]
async fn telegram_media_group_retries_transient_failure_before_success() -> Result<()> {
    let Some((api_base, state, handle)) =
        spawn_mock_telegram_media_api_with_group_failure(1).await?
    else {
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
            "[IMAGE:https://example.com/a.png][DOCUMENT:https://example.com/b.pdf]",
            "123456",
        )
        .await?;

    let calls = state.calls.lock().await;
    let methods: Vec<&str> = calls.iter().map(|c| c.method.as_str()).collect();
    assert_eq!(methods, vec!["sendMediaGroup", "sendMediaGroup"]);

    handle.abort();
    Ok(())
}

#[tokio::test]
async fn telegram_media_group_falls_back_to_sequential_on_group_failure() -> Result<()> {
    let Some((api_base, state, handle)) =
        spawn_mock_telegram_media_api_with_group_failure(3).await?
    else {
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
            "[IMAGE:https://example.com/a.png][DOCUMENT:https://example.com/b.pdf]",
            "123456",
        )
        .await?;

    let calls = state.calls.lock().await;
    let methods: Vec<&str> = calls.iter().map(|c| c.method.as_str()).collect();
    assert_eq!(
        methods,
        vec![
            "sendMediaGroup",
            "sendMediaGroup",
            "sendMediaGroup",
            "sendPhoto",
            "sendDocument"
        ]
    );

    handle.abort();
    Ok(())
}

#[tokio::test]
async fn telegram_media_marker_keeps_invalid_marker_as_plain_text() -> Result<()> {
    let Some((api_base, state, handle)) = spawn_mock_telegram_media_api().await? else {
        return Ok(());
    };

    let channel = TelegramChannel::new_with_base_url(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        api_base,
    );

    channel
        .send("keep [IMAGE:not-a-url] text", "123456")
        .await?;

    let calls = state.calls.lock().await;
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].method, "sendMessage");
    assert_eq!(
        calls[0]
            .payload
            .get("text")
            .and_then(serde_json::Value::as_str),
        Some("keep [IMAGE:not-a-url] text")
    );

    handle.abort();
    Ok(())
}
