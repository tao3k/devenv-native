use super::support::{DiscordChannel, discord_slash_interaction_event, parse_message};

#[test]
fn discord_parse_gateway_message_parses_slash_interaction_as_command_text() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["alice".to_string()], vec![]);
    let event = discord_slash_interaction_event(
        "9001",
        "session",
        "2001",
        Some("3001"),
        "1001",
        "alice",
        serde_json::json!([
            {
                "name": "memory",
                "type": 1,
                "options": [
                    {
                        "name": "format",
                        "type": 3,
                        "value": "json"
                    }
                ]
            }
        ]),
        2,
    );

    let parsed = parse_message!(channel, &event, "slash interaction should parse");
    assert_eq!(parsed.sender, "1001");
    assert_eq!(parsed.content, "/session memory json");
    assert_eq!(parsed.session_key, "3001:2001:1001");
}

#[test]
fn discord_parse_gateway_message_parses_slash_prompt_option_with_spaces() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["*".to_string()], vec![]);
    let event = discord_slash_interaction_event(
        "9002",
        "bg",
        "2001",
        Some("3001"),
        "1001",
        "alice",
        serde_json::json!([
            {
                "name": "prompt",
                "type": 3,
                "value": "collect logs and summarize failures"
            }
        ]),
        2,
    );

    let parsed = parse_message!(channel, &event, "bg interaction should parse");
    assert_eq!(parsed.content, "/bg collect logs and summarize failures");
}

#[test]
fn discord_parse_gateway_message_ignores_non_command_interaction_payload() {
    let channel = DiscordChannel::new("fake-token".to_string(), vec!["*".to_string()], vec![]);
    let event = discord_slash_interaction_event(
        "9003",
        "session",
        "2001",
        Some("3001"),
        "1001",
        "alice",
        serde_json::json!([]),
        4,
    );
    assert!(channel.parse_gateway_message(&event).is_none());
}
