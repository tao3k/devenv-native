use xiuxian_daochang::Channel;

use super::support::{DiscordChannel, DiscordSessionPartition, discord_event, parse_message};

#[test]
fn discord_parse_gateway_message_defaults_dm_scope() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["*".to_string()], vec![]);
    let event = discord_event("1", "hello", "2001", None, "1001", Some("alice"));

    let parsed = parse_message!(channel, &event, "message should parse");
    assert_eq!(parsed.session_key, "dm:2001:1001");
}

#[test]
fn discord_parse_gateway_message_partition_channel_only() {
    let channel = DiscordChannel::new_with_partition(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        DiscordSessionPartition::ChannelOnly,
    );
    let event_a = discord_event("1", "hello", "2001", Some("3001"), "1001", Some("alice"));
    let event_b = discord_event("2", "hello", "2001", Some("3001"), "1002", Some("bob"));

    let parsed_a = parse_message!(channel, &event_a, "message A should parse");
    let parsed_b = parse_message!(channel, &event_b, "message B should parse");
    assert_eq!(parsed_a.session_key, "3001:2001");
    assert_eq!(parsed_a.session_key, parsed_b.session_key);
}

#[test]
fn discord_parse_gateway_message_partition_user_only() {
    let channel = DiscordChannel::new_with_partition(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        DiscordSessionPartition::UserOnly,
    );
    let event_a = discord_event("1", "hello", "2001", Some("3001"), "1001", Some("alice"));
    let event_b = discord_event("2", "hello", "2002", Some("3001"), "1001", Some("alice"));

    let parsed_a = parse_message!(channel, &event_a, "message A should parse");
    let parsed_b = parse_message!(channel, &event_b, "message B should parse");
    assert_eq!(parsed_a.session_key, "1001");
    assert_eq!(parsed_a.session_key, parsed_b.session_key);
}

#[test]
fn discord_parse_gateway_message_partition_guild_user() {
    let channel = DiscordChannel::new_with_partition(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        DiscordSessionPartition::GuildUser,
    );
    let event_a = discord_event("1", "hello", "2001", Some("3001"), "1001", Some("alice"));
    let event_b = discord_event("2", "hello", "2002", Some("3001"), "1001", Some("alice"));
    let event_c = discord_event("3", "hello", "2003", Some("3002"), "1001", Some("alice"));

    let parsed_a = parse_message!(channel, &event_a, "message A should parse");
    let parsed_b = parse_message!(channel, &event_b, "message B should parse");
    let parsed_c = parse_message!(channel, &event_c, "message C should parse");
    assert_eq!(parsed_a.session_key, "3001:1001");
    assert_eq!(parsed_a.session_key, parsed_b.session_key);
    assert_ne!(parsed_a.session_key, parsed_c.session_key);
}

#[test]
fn discord_session_partition_runtime_toggle_changes_strategy() {
    let channel = DiscordChannel::new_with_partition(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        DiscordSessionPartition::GuildChannelUser,
    );
    let event_a = discord_event("1", "hello", "2001", Some("3001"), "1001", Some("alice"));
    let event_b = discord_event("2", "hello", "2001", Some("3001"), "1002", Some("bob"));

    let parsed_a = parse_message!(channel, &event_a, "message A should parse");
    let parsed_b = parse_message!(channel, &event_b, "message B should parse");
    assert_ne!(parsed_a.session_key, parsed_b.session_key);

    if let Err(error) = channel.set_session_partition_mode("channel") {
        panic!("mode should be accepted: {error}");
    }

    let parsed_a_shared = parse_message!(channel, &event_a, "message A shared should parse");
    let parsed_b_shared = parse_message!(channel, &event_b, "message B shared should parse");
    assert_eq!(parsed_a_shared.session_key, "3001:2001");
    assert_eq!(parsed_a_shared.session_key, parsed_b_shared.session_key);
}

#[test]
fn discord_session_partition_mode_rejects_invalid_value() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["*".to_string()], vec![]);
    let error = match channel.set_session_partition_mode("invalid") {
        Ok(()) => panic!("invalid mode should fail"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("invalid discord session partition mode")
    );
}
