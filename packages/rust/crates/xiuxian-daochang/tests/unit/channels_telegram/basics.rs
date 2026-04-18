use super::support::{
    Channel, TelegramChannel, group_photo_update, group_text_update, require_some,
};

#[test]
fn telegram_channel_name() {
    let channel = TelegramChannel::new("fake-token".into(), vec!["*".into()], vec![]);
    assert_eq!(channel.name(), "telegram");
}

#[test]
fn telegram_parse_update_builds_group_chat_session_key_by_default() {
    let channel = TelegramChannel::new("t".into(), vec!["*".into()], vec![]);
    let message = require_some(
        channel.parse_update_message(&group_text_update(-200_123, 888, "hello")),
        "message",
    );

    assert_eq!(message.recipient, "-200123");
    assert_eq!(message.sender, "888");
    assert_eq!(message.session_key, "-200123");
    assert_eq!(message.content, "hello");
}

#[test]
fn telegram_parse_update_uses_caption_for_photo_messages() {
    let channel = TelegramChannel::new("t".into(), vec!["*".into()], vec![]);
    let message = require_some(
        channel.parse_update_message(&group_photo_update(
            -200_123,
            888,
            Some("please analyze this image"),
        )),
        "message",
    );

    assert_eq!(message.content, "please analyze this image");
}

#[test]
fn telegram_parse_update_generates_placeholder_for_photo_without_caption() {
    let channel = TelegramChannel::new("t".into(), vec!["*".into()], vec![]);
    let message = require_some(
        channel.parse_update_message(&group_photo_update(-200_123, 888, None)),
        "message",
    );

    assert_eq!(message.content, "[telegram-photo]");
}

#[test]
fn telegram_parse_update_rejects_unauthorized_user() {
    let channel = TelegramChannel::new("t".into(), vec!["999".into()], vec![]);
    assert!(
        channel
            .parse_update_message(&group_text_update(-200_123, 888, "hello"))
            .is_none()
    );
}

#[test]
fn telegram_parse_update_rejects_all_when_allowlist_empty() {
    let channel = TelegramChannel::new("t".into(), vec![], vec![]);
    assert!(
        channel
            .parse_update_message(&group_text_update(-200_123, 888, "hello"))
            .is_none()
    );
}
