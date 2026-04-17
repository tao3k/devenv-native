use super::support::{
    build_channel_with_settings, group_update, group_update_for_chat, topic_update,
};

#[test]
fn telegram_group_policy_topic_override_has_higher_priority() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
telegram:
  acl:
    allow:
      users: []
      groups: ["-200100"]
  group_policy: "open"
  groups:
    "-200100":
      topics:
        "42":
          group_policy: "allowlist"
          allow_from:
            users: ["111"]
"#,
    );

    assert!(
        channel
            .parse_update_message(&topic_update(111, "topic hello", 42))
            .is_some()
    );
    assert!(
        channel
            .parse_update_message(&topic_update(222, "topic hello", 42))
            .is_none()
    );
    assert!(
        channel
            .parse_update_message(&topic_update(222, "other topic", 99))
            .is_some()
    );
}

#[test]
fn telegram_group_policy_wildcard_override_is_applied_before_specific_group_override() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
telegram:
  acl:
    allow:
      users: []
      groups: ["-200100", "-200200"]
  group_policy: "open"
  require_mention: false
  groups:
    "*":
      require_mention: true
    "-200100":
      require_mention: false
"#,
    );

    let group_specific = group_update_for_chat(-200100, 111, "plain group text");
    assert!(channel.parse_update_message(&group_specific).is_some());

    let wildcard_only = group_update_for_chat(-200200, 111, "plain group text");
    assert!(channel.parse_update_message(&wildcard_only).is_none());

    let wildcard_triggered = group_update_for_chat(-200200, 111, "/session status");
    assert!(channel.parse_update_message(&wildcard_triggered).is_some());
}

#[test]
fn telegram_group_policy_require_mention_accepts_reply_to_bot_trigger() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
telegram:
  acl:
    allow:
      users: []
      groups: ["-200100"]
  group_policy: "open"
  require_mention: true
"#,
    );

    let mut update = group_update(111, "hello in reply");
    update["message"]["reply_to_message"] = serde_json::json!({
      "from": { "is_bot": true }
    });
    assert!(channel.parse_update_message(&update).is_some());
}

#[test]
fn telegram_group_policy_require_mention_accepts_entity_mention_trigger() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
telegram:
  acl:
    allow:
      users: []
      groups: ["-200100"]
  group_policy: "open"
  require_mention: true
"#,
    );

    let mut update = group_update(111, "hello");
    update["message"]["entities"] = serde_json::json!([
      { "type": "mention", "offset": 0, "length": 5 }
    ]);
    assert!(channel.parse_update_message(&update).is_some());
}
