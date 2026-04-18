use std::sync::Arc;
use std::time::{Duration, Instant};

use super::support::{
    Channel, Result, TELEGRAM_MAX_MESSAGE_LENGTH, TelegramChannel, TelegramSessionPartition,
    anyhow, decorate_chunk_for_telegram, markdown_to_telegram_html,
    markdown_to_telegram_markdown_v2, spawn_delayed_send_mock_telegram_api,
    spawn_mock_telegram_api, spawn_rate_limit_gate_mock_telegram_api,
    spawn_retry_then_success_mock_telegram_api, split_message_for_telegram,
};

#[tokio::test]
async fn telegram_send_preserves_full_text_when_markdown_escaping_would_overflow() -> Result<()> {
    let Some((api_base, state, handle)) = spawn_mock_telegram_api(None).await? else {
        return Ok(());
    };
    let channel = TelegramChannel::new_with_base_url(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        api_base,
    );

    let message = "!".repeat(9000);
    channel.send(&message, "123456").await?;

    let requests = state.requests.lock().await;
    let chunks = split_message_for_telegram(&message);
    assert!(
        chunks.iter().enumerate().any(|(index, chunk)| {
            let plain = decorate_chunk_for_telegram(chunk, index, chunks.len());
            markdown_to_telegram_markdown_v2(&plain).chars().count() > TELEGRAM_MAX_MESSAGE_LENGTH
        }),
        "test precondition failed: at least one chunk must overflow MarkdownV2 limit"
    );
    assert_eq!(requests.len(), chunks.len());

    for (index, request) in requests.iter().enumerate() {
        let plain_chunk = decorate_chunk_for_telegram(&chunks[index], index, chunks.len());
        let markdown_chunk = markdown_to_telegram_markdown_v2(&plain_chunk);
        let html_chunk = markdown_to_telegram_html(&plain_chunk);
        let markdown_overflow = markdown_chunk.chars().count() > TELEGRAM_MAX_MESSAGE_LENGTH;
        let html_overflow = html_chunk.chars().count() > TELEGRAM_MAX_MESSAGE_LENGTH;
        let prefer_html = markdown_chunk
            .chars()
            .count()
            .saturating_sub(html_chunk.chars().count())
            >= 256;
        let parse_mode = request
            .get("parse_mode")
            .and_then(serde_json::Value::as_str);
        let text = request
            .get("text")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();

        if markdown_overflow && !html_overflow {
            assert_eq!(parse_mode, Some("HTML"));
            assert_eq!(text, html_chunk);
        } else if markdown_overflow && html_overflow {
            assert!(parse_mode.is_none());
            assert_eq!(text, plain_chunk);
        } else if prefer_html && !html_overflow {
            assert_eq!(parse_mode, Some("HTML"));
            assert_eq!(text, html_chunk);
        } else {
            assert_eq!(parse_mode, Some("MarkdownV2"));
            assert_eq!(text, markdown_chunk);
        }

        assert!(plain_chunk.chars().count() <= TELEGRAM_MAX_MESSAGE_LENGTH);
    }

    handle.abort();
    Ok(())
}

#[tokio::test]
async fn telegram_send_truncates_very_large_payload_to_prevent_flood() -> Result<()> {
    let Some((api_base, state, handle)) = spawn_mock_telegram_api(None).await? else {
        return Ok(());
    };
    let channel = TelegramChannel::new_with_base_url(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        api_base,
    );

    let chunk_chars = TELEGRAM_MAX_MESSAGE_LENGTH - xiuxian_daochang::chunk_marker_reserve_chars();
    let message = "x".repeat(chunk_chars * 40);
    let expected_chunks = split_message_for_telegram(&message).len();
    assert!(
        expected_chunks > 32,
        "precondition: payload should exceed auto-chunk guard threshold"
    );

    channel.send(&message, "123456").await?;

    let requests = state.requests.lock().await;
    assert!(
        requests.len() < expected_chunks,
        "output guard should reduce sent chunks"
    );
    let last = requests
        .last()
        .and_then(|request| request.get("text"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    assert!(
        last.contains("Output truncated after"),
        "last message should announce truncation guard"
    );

    handle.abort();
    Ok(())
}

#[tokio::test]
async fn telegram_send_retries_on_rate_limit_and_succeeds() -> Result<()> {
    let Some((api_base, state, handle)) = spawn_retry_then_success_mock_telegram_api(1).await?
    else {
        return Ok(());
    };
    let channel = TelegramChannel::new_with_base_url(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        api_base,
    );

    channel.send("retry check", "123456").await?;

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
        Some("MarkdownV2")
    );

    handle.abort();
    Ok(())
}

#[tokio::test]
async fn telegram_send_global_rate_limit_gate_delays_parallel_send() -> Result<()> {
    let Some((api_base, state, handle)) = spawn_rate_limit_gate_mock_telegram_api().await? else {
        return Ok(());
    };
    let channel = Arc::new(TelegramChannel::new_with_base_url(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        api_base,
    ));

    let first_channel = Arc::clone(&channel);
    let first_send =
        tokio::spawn(async move { first_channel.send("firstgatecheck", "123456").await });

    for _ in 0..50 {
        if *state.first_rate_limit_emitted.lock().await {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    let second_channel = Arc::clone(&channel);
    let second_send =
        tokio::spawn(async move { second_channel.send("secondgatecheck", "123456").await });

    first_send.await??;
    second_send.await??;

    let requests = state.requests.lock().await;
    let first_request_at = requests
        .iter()
        .find_map(|request| {
            (request
                .payload
                .get("text")
                .and_then(serde_json::Value::as_str)
                == Some("firstgatecheck"))
            .then_some(request.received_at)
        })
        .ok_or_else(|| anyhow!("first send request should be captured"))?;
    let second_request_at = requests
        .iter()
        .find_map(|request| {
            (request
                .payload
                .get("text")
                .and_then(serde_json::Value::as_str)
                == Some("secondgatecheck"))
            .then_some(request.received_at)
        })
        .ok_or_else(|| anyhow!("second send request should be captured"))?;
    let wait_before_second_request = second_request_at
        .checked_duration_since(first_request_at)
        .ok_or_else(|| anyhow!("second request timestamp should not precede first request"))?;
    assert!(
        wait_before_second_request >= Duration::from_millis(850),
        "expected second send to wait for global retry window after first rate-limit response, got {}ms",
        wait_before_second_request.as_millis()
    );

    let first_request_count = requests
        .iter()
        .filter(|request| {
            request
                .payload
                .get("text")
                .and_then(serde_json::Value::as_str)
                == Some("firstgatecheck")
        })
        .count();
    assert_eq!(first_request_count, 2);

    handle.abort();
    Ok(())
}

#[tokio::test]
async fn telegram_send_global_rate_limit_gate_spreads_parallel_followup_requests() -> Result<()> {
    let Some((api_base, state, handle)) = spawn_rate_limit_gate_mock_telegram_api().await? else {
        return Ok(());
    };
    let channel = Arc::new(TelegramChannel::new_with_base_url(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        api_base,
    ));

    let first_channel = Arc::clone(&channel);
    let first_send =
        tokio::spawn(async move { first_channel.send("firstgatecheck", "123456").await });

    for _ in 0..50 {
        if *state.first_rate_limit_emitted.lock().await {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    let second_channel = Arc::clone(&channel);
    let second_send =
        tokio::spawn(async move { second_channel.send("secondspreadcheck", "123456").await });

    let third_channel = Arc::clone(&channel);
    let third_send =
        tokio::spawn(async move { third_channel.send("thirdspreadcheck", "123456").await });

    first_send.await??;
    second_send.await??;
    third_send.await??;

    let requests = state.requests.lock().await;
    let mut followup_times = requests
        .iter()
        .filter_map(|request| {
            let text = request
                .payload
                .get("text")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            match text {
                "secondspreadcheck" | "thirdspreadcheck" => Some(request.received_at),
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(followup_times.len(), 2);
    followup_times.sort_unstable();
    let spread_gap = followup_times[1].duration_since(followup_times[0]);
    assert!(
        spread_gap >= Duration::from_millis(30),
        "expected staggered follow-up requests after rate limit gate, gap={}ms",
        spread_gap.as_millis()
    );

    handle.abort();
    Ok(())
}

#[tokio::test]
async fn telegram_send_returns_timeout_error_for_slow_http_response() -> Result<()> {
    let Some((api_base, handle)) =
        spawn_delayed_send_mock_telegram_api(Duration::from_millis(250)).await?
    else {
        return Ok(());
    };

    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_millis(50))
        .timeout(Duration::from_millis(50))
        .build()?;
    let channel = TelegramChannel::new_with_base_url_and_partition_and_client(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        api_base,
        vec!["*".to_string()],
        TelegramSessionPartition::ChatUser,
        client,
    );

    let started_at = Instant::now();
    let result = channel.send("timeout check", "123456").await;
    let Err(error) = result else {
        return Err(anyhow!(
            "send should time out with a very short client timeout"
        ));
    };
    assert!(
        started_at.elapsed() < Duration::from_secs(2),
        "send should fail quickly when request timeout is configured"
    );
    let error_message = error.to_string().to_lowercase();
    assert!(
        error_message.contains("timed out") || error_message.contains("deadline has elapsed"),
        "expected timeout error, got: {error}"
    );

    handle.abort();
    Ok(())
}
