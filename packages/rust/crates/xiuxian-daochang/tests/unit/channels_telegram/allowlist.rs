use super::support::{
    TelegramChannel, group_text_update, group_text_update_with_title, require_some,
};

#[test]
fn telegram_parse_update_allows_numeric_user_id_in_allowlist() {
    let channel = TelegramChannel::new("t".into(), vec!["888".into()], vec![]);
    let message = require_some(
        channel.parse_update_message(&group_text_update(-200_123, 888, "hello")),
        "message",
    );

    assert_eq!(message.sender, "888");
}

#[test]
fn telegram_parse_update_allows_prefixed_numeric_user_id_in_allowlist() {
    let channel = TelegramChannel::new("t".into(), vec!["telegram:888".into()], vec![]);
    let message = require_some(
        channel.parse_update_message(&group_text_update(-200_123, 888, "hello")),
        "message",
    );

    assert_eq!(message.sender, "888");
}

#[test]
fn telegram_parse_update_rejects_username_allowlist_entries() {
    let channel = TelegramChannel::new("t".into(), vec!["@alice".into()], vec![]);
    assert!(
        channel
            .parse_update_message(&group_text_update(-200_123, 888, "hello"))
            .is_none()
    );
}

#[test]
fn telegram_parse_update_ignores_invalid_allowlist_entries_and_keeps_numeric_entries() {
    let channel = TelegramChannel::new("t".into(), vec!["@alice".into(), "888".into()], vec![]);
    let message = require_some(
        channel.parse_update_message(&group_text_update(-200_123, 888, "hello")),
        "message",
    );

    assert_eq!(message.sender, "888");
}

#[test]
fn telegram_parse_update_trims_allowlist_entries() {
    let channel = TelegramChannel::new(
        "t".into(),
        vec!["  tg:888  ".into(), " 888 ".into()],
        vec![],
    );
    let message = require_some(
        channel.parse_update_message(&group_text_update(-200_123, 888, "hello")),
        "message",
    );

    assert_eq!(message.sender, "888");
}

#[test]
fn telegram_parse_update_allows_message_from_allowed_group() {
    let channel = TelegramChannel::new("t".into(), vec![], vec!["-200123".into()]);
    let message = require_some(
        channel.parse_update_message(&group_text_update(-200_123, 999, "hi from group")),
        "message",
    );

    assert_eq!(message.recipient, "-200123");
    assert_eq!(message.sender, "999");
    assert_eq!(message.session_key, "-200123");
}

#[test]
fn telegram_parse_update_allows_message_from_allowed_group_with_chat_title() {
    let channel = TelegramChannel::new("t".into(), vec![], vec!["-200123".into()]);
    let message = require_some(
        channel.parse_update_message(&group_text_update_with_title(
            -200_123,
            999,
            "hi from group",
            "Test1",
        )),
        "message",
    );

    assert_eq!(message.recipient, "-200123");
    assert_eq!(message.sender, "999");
    assert_eq!(message.session_key, "-200123");
}
