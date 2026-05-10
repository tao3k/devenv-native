use super::support::{
    DiscordChannel, discord_event, discord_event_reply_to, discord_event_with_mentions,
    parse_message,
};

#[test]
fn discord_parse_gateway_message_require_mention_blocks_plain_guild_text() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["*".to_string()], vec![]);
    channel.set_bot_user_id_for_tests(Some("9999".into()));
    channel.configure_mention_policy_for_tests(true, std::collections::HashMap::new());
    let event = discord_event("1", "hello", "2001", Some("3001"), "1001", Some("alice"));

    assert!(channel.parse_gateway_message(&event).is_none());
}

#[test]
fn discord_parse_gateway_message_require_mention_accepts_bot_mention() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["*".to_string()], vec![]);
    channel.set_bot_user_id_for_tests(Some("9999".into()));
    channel.configure_mention_policy_for_tests(true, std::collections::HashMap::new());
    let event = discord_event_with_mentions(
        "1",
        "<@9999> hello",
        "2001",
        Some("3001"),
        "1001",
        Some("alice"),
        &["9999"],
    );

    let parsed = parse_message!(channel, &event, "mention-triggered message should parse");
    assert_eq!(parsed.content, "<@9999> hello");
}

#[test]
fn discord_parse_gateway_message_require_mention_accepts_reply_to_bot() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["*".to_string()], vec![]);
    channel.set_bot_user_id_for_tests(Some("9999".into()));
    channel.configure_mention_policy_for_tests(true, std::collections::HashMap::new());
    let event = discord_event_reply_to(
        "1",
        "continuing thread",
        "2001",
        Some("3001"),
        "1001",
        Some("alice"),
        "9999",
    );

    let parsed = parse_message!(channel, &event, "reply-to-bot message should parse");
    assert_eq!(parsed.content, "continuing thread");
}

#[test]
fn discord_parse_gateway_message_require_mention_accepts_command_without_mention() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["*".to_string()], vec![]);
    channel.set_bot_user_id_for_tests(Some("9999".into()));
    channel.configure_mention_policy_for_tests(true, std::collections::HashMap::new());
    let event = discord_event(
        "1",
        "/session mention off",
        "2001",
        Some("3001"),
        "1001",
        Some("alice"),
    );

    let parsed = parse_message!(channel, &event, "slash-style control command should parse");
    assert_eq!(parsed.content, "/session mention off");
}

#[test]
fn discord_parse_gateway_message_channel_override_can_disable_require_mention() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["*".to_string()], vec![]);
    channel.set_bot_user_id_for_tests(Some("9999".into()));
    let mut overrides = std::collections::HashMap::new();
    overrides.insert("2001".to_string(), false);
    channel.configure_mention_policy_for_tests(true, overrides);
    let event = discord_event("1", "hello", "2001", Some("3001"), "1001", Some("alice"));

    let parsed = parse_message!(channel, &event, "channel override should open parsing");
    assert_eq!(parsed.recipient, "2001");
}
