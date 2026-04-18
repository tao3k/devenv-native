use super::support::{
    TelegramChannel, TelegramSessionPartition, group_text_update, group_text_update_with_thread,
    require_some,
};

#[test]
fn telegram_parse_update_partition_chat_only() {
    let channel = TelegramChannel::new_with_partition(
        "t".into(),
        vec!["*".into()],
        vec![],
        TelegramSessionPartition::ChatOnly,
    );

    let message = require_some(
        channel.parse_update_message(&group_text_update(-200_123, 1001, "chat scope")),
        "message",
    );
    assert_eq!(message.session_key, "-200123");
}

#[test]
fn telegram_parse_update_partition_chat_only_isolates_different_chats() {
    let channel = TelegramChannel::new_with_partition(
        "t".into(),
        vec!["*".into()],
        vec![],
        TelegramSessionPartition::ChatOnly,
    );

    let message_a = require_some(
        channel.parse_update_message(&group_text_update(-200_111, 1001, "chat scope A")),
        "message A",
    );
    let message_b = require_some(
        channel.parse_update_message(&group_text_update(-200_222, 1001, "chat scope B")),
        "message B",
    );
    assert_eq!(message_a.session_key, "-200111");
    assert_eq!(message_b.session_key, "-200222");
    assert_ne!(message_a.session_key, message_b.session_key);
}

#[test]
fn telegram_parse_update_partition_user_only() {
    let channel = TelegramChannel::new_with_partition(
        "t".into(),
        vec!["*".into()],
        vec![],
        TelegramSessionPartition::UserOnly,
    );

    let message = require_some(
        channel.parse_update_message(&group_text_update(-200_999, 1001, "user scope")),
        "message",
    );
    assert_eq!(message.session_key, "1001");
}

#[test]
fn telegram_parse_update_partition_chat_thread_user() {
    let channel = TelegramChannel::new_with_partition(
        "t".into(),
        vec!["*".into()],
        vec![],
        TelegramSessionPartition::ChatThreadUser,
    );

    let message = require_some(
        channel.parse_update_message(&group_text_update_with_thread(
            -200_123,
            1001,
            "thread scope",
            42,
        )),
        "message",
    );
    assert_eq!(message.session_key, "-200123:42:1001");
    assert_eq!(message.recipient, "-200123:42");
}

#[test]
fn telegram_parse_update_partition_runtime_toggle_changes_session_key_strategy() {
    let channel = TelegramChannel::new_with_partition(
        "t".into(),
        vec!["*".into()],
        vec![],
        TelegramSessionPartition::ChatUser,
    );

    let update_a = group_text_update(-200_111, 1001, "hello");
    let update_b = group_text_update(-200_111, 1002, "hello");
    let message_a = require_some(channel.parse_update_message(&update_a), "message A");
    let message_b = require_some(channel.parse_update_message(&update_b), "message B");
    assert_ne!(message_a.session_key, message_b.session_key);

    channel.set_session_partition(TelegramSessionPartition::ChatOnly);

    let shared_from_alice =
        require_some(channel.parse_update_message(&update_a), "message A shared");
    let shared_from_bob = require_some(channel.parse_update_message(&update_b), "message B shared");
    assert_eq!(shared_from_alice.session_key, "-200111");
    assert_eq!(shared_from_alice.session_key, shared_from_bob.session_key);
}

#[test]
fn telegram_session_partition_parse_aliases() {
    assert_eq!(
        "chat_user".parse::<TelegramSessionPartition>().ok(),
        Some(TelegramSessionPartition::ChatUser)
    );
    assert_eq!(
        "chat".parse::<TelegramSessionPartition>().ok(),
        Some(TelegramSessionPartition::ChatOnly)
    );
    assert_eq!(
        "user".parse::<TelegramSessionPartition>().ok(),
        Some(TelegramSessionPartition::UserOnly)
    );
    assert_eq!(
        "topic-user".parse::<TelegramSessionPartition>().ok(),
        Some(TelegramSessionPartition::ChatThreadUser)
    );
    assert!("invalid".parse::<TelegramSessionPartition>().is_err());
}
