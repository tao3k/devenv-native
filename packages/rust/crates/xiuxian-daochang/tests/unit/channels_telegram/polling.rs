use std::time::Duration;

use super::support::{
    Channel, ChannelAttachment, Result, TelegramChannel, anyhow, require_some,
    spawn_mock_telegram_polling_media_api,
};

#[tokio::test]
async fn telegram_listen_enriches_photo_message_with_image_attachment() -> Result<()> {
    let Some((api_base, state, server_handle)) = spawn_mock_telegram_polling_media_api().await?
    else {
        return Ok(());
    };

    let channel = TelegramChannel::new_with_base_url(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        api_base.clone(),
    );
    let (tx, mut rx) = tokio::sync::mpsc::channel(8);
    let listen_task = tokio::spawn(async move {
        let _ = channel.listen(tx).await;
    });

    let maybe_message = tokio::time::timeout(Duration::from_secs(3), rx.recv())
        .await
        .map_err(|error| anyhow!("timed out waiting for inbound telegram message: {error}"))?;
    let message = require_some(maybe_message, "inbound telegram message");

    let expected_url = format!("{api_base}/file/botfake-token/photos/vision.jpg");
    assert_eq!(message.content, "please analyze this image");
    assert!(matches!(
        message.attachments.first(),
        Some(ChannelAttachment::ImageUrl { url }) if url == expected_url.as_str()
    ));

    let get_file_requests = state.get_file_requests.lock().await;
    assert_eq!(get_file_requests.len(), 1);
    assert_eq!(
        get_file_requests[0]
            .get("file_id")
            .and_then(serde_json::Value::as_str),
        Some("photo_file_large")
    );

    listen_task.abort();
    server_handle.abort();
    Ok(())
}
