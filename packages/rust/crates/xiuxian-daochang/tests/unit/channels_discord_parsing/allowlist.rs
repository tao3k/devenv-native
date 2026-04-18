use super::support::{DiscordChannel, discord_event, discord_event_with_roles, parse_message};

#[test]
fn discord_parse_gateway_message_allows_allowed_user() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["alice".to_string()], vec![]);
    let event = discord_event("1", "hello", "2001", Some("3001"), "1001", Some("alice"));

    let parsed = parse_message!(channel, &event, "message should parse");
    assert_eq!(parsed.sender, "1001");
    assert_eq!(parsed.recipient, "2001");
    assert_eq!(parsed.channel, "discord");
}

#[test]
fn discord_parse_gateway_message_allows_allowed_guild() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec![], vec!["3001".to_string()]);
    let event = discord_event("1", "hello", "2001", Some("3001"), "1001", Some("unknown"));

    let parsed = parse_message!(channel, &event, "message should parse");
    assert_eq!(parsed.sender, "1001");
    assert_eq!(parsed.session_key, "3001:2001:1001");
}

#[test]
fn discord_parse_gateway_message_allows_allowed_role_identity() {
    let channel = DiscordChannel::new(
        "fake-token".to_string(),
        vec!["role:9001".to_string()],
        vec![],
    );
    let event = discord_event_with_roles(
        "1",
        "hello",
        "2001",
        Some("3001"),
        "1001",
        Some("alice"),
        &["9001"],
    );

    let parsed = parse_message!(channel, &event, "message should parse");
    assert_eq!(parsed.sender, "1001");
    assert_eq!(parsed.recipient, "2001");
}

#[test]
fn discord_parse_gateway_message_rejects_unauthorized_sender() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["owner".to_string()], vec![]);
    let event = discord_event("1", "hello", "2001", Some("3001"), "1001", Some("alice"));

    assert!(channel.parse_gateway_message(&event).is_none());
}

#[test]
fn discord_parse_gateway_message_rejects_empty_content() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["*".to_string()], vec![]);
    let event = discord_event("1", "   ", "2001", Some("3001"), "1001", Some("alice"));

    assert!(channel.parse_gateway_message(&event).is_none());
}

#[test]
fn discord_parse_gateway_message_rejects_invalid_snowflake_payload() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["*".to_string()], vec![]);
    let event = discord_event(
        "not-a-snowflake",
        "hello",
        "2001",
        Some("3001"),
        "1001",
        Some("alice"),
    );

    assert!(channel.parse_gateway_message(&event).is_none());
}
