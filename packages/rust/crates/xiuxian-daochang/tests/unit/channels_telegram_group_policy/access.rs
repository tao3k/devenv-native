use super::support::{build_channel_with_settings, group_update};

#[test]
fn telegram_group_policy_disabled_rejects_group_messages() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
telegram:
  acl:
    allow:
      users: []
      groups: ["-200100"]
  group_policy: "disabled"
"#,
    );

    assert!(
        channel
            .parse_update_message(&group_update(111, "hello"))
            .is_none()
    );
}

#[test]
fn telegram_group_policy_allowlist_uses_group_allow_from() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
telegram:
  acl:
    allow:
      users: []
      groups: ["-200100"]
  group_policy: "allowlist"
  group_allow_from: "111"
"#,
    );

    assert!(
        channel
            .parse_update_message(&group_update(111, "hello"))
            .is_some()
    );
    assert!(
        channel
            .parse_update_message(&group_update(222, "hello"))
            .is_none()
    );
}

#[test]
fn telegram_group_policy_allowlist_falls_back_to_allowed_users_when_group_allow_from_unset() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
telegram:
  acl:
    allow:
      users: ["999"]
      groups: ["-200100"]
  group_policy: "allowlist"
"#,
    );

    assert!(
        channel
            .parse_update_message(&group_update(999, "hello"))
            .is_some()
    );
    assert!(
        channel
            .parse_update_message(&group_update(111, "hello"))
            .is_none()
    );
}

#[test]
fn telegram_group_policy_group_override_can_open_when_global_disabled() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
telegram:
  acl:
    allow:
      users: []
      groups: ["-200100"]
  group_policy: "disabled"
  groups:
    "-200100":
      group_policy: "open"
"#,
    );

    assert!(
        channel
            .parse_update_message(&group_update(333, "hello"))
            .is_some()
    );
}

#[test]
fn telegram_group_policy_require_mention_blocks_plain_group_text() {
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

    assert!(
        channel
            .parse_update_message(&group_update(111, "hello everyone"))
            .is_none()
    );
    assert!(
        channel
            .parse_update_message(&group_update(111, "/session status"))
            .is_some()
    );
    assert!(
        channel
            .parse_update_message(&group_update(111, "@bot hello"))
            .is_some()
    );
}
