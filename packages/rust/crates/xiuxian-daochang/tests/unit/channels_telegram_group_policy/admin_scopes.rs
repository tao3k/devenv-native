use super::support::{Channel, build_channel_with_settings};

#[test]
fn telegram_group_policy_group_admin_users_are_scoped_by_recipient() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
telegram:
  acl:
    allow:
      users: []
      groups: ["-200100", "-200200"]
    admin:
      users: []
  groups:
    "-200100":
      admin_users:
        users: ["111"]
"#,
    );

    assert!(channel.is_authorized_for_control_command_for_recipient(
        "telegram:111",
        "/reset",
        "-200100",
    ));
    assert!(!channel.is_authorized_for_control_command_for_recipient("111", "/reset", "-200200",));
    assert!(!channel.is_authorized_for_control_command_for_recipient("111", "/reset", "12345",));
    assert!(channel.is_authorized_for_slash_command_for_recipient(
        "111",
        "session.status",
        "-200100",
    ));
    assert!(!channel.is_authorized_for_slash_command_for_recipient(
        "111",
        "session.status",
        "-200200",
    ));
}

#[test]
fn telegram_group_policy_topic_admin_users_override_group_and_wildcard_admin_users() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
telegram:
  acl:
    allow:
      users: []
      groups: ["-200100", "-200200"]
    admin:
      users: []
  groups:
    "*":
      admin_users:
        users: ["900"]
    "-200100":
      admin_users:
        users: ["111"]
      topics:
        "42":
          admin_users:
            users: ["222"]
"#,
    );

    assert!(channel.is_authorized_for_control_command_for_recipient(
        "222",
        "/session partition",
        "-200100:42",
    ));
    assert!(!channel.is_authorized_for_control_command_for_recipient(
        "111",
        "/session partition",
        "-200100:42",
    ));
    assert!(channel.is_authorized_for_control_command_for_recipient(
        "111",
        "/session partition",
        "-200100:99",
    ));
    assert!(channel.is_authorized_for_control_command_for_recipient(
        "900",
        "/session partition",
        "-200200",
    ));
    assert!(channel.is_authorized_for_slash_command_for_recipient(
        "222",
        "session.memory",
        "-200100:42",
    ));
}

#[test]
fn telegram_group_policy_group_admin_users_do_not_override_explicit_global_control_deny() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
telegram:
  acl:
    allow:
      users: []
      groups: ["-200100"]
    admin:
      users: []
    control:
      allow_from:
        users: []
  groups:
    "-200100":
      admin_users:
        users: ["111"]
"#,
    );

    assert!(!channel.is_authorized_for_control_command_for_recipient("111", "/reset", "-200100",));
}

#[test]
fn telegram_group_policy_group_admin_users_do_not_override_explicit_global_slash_deny() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
telegram:
  acl:
    allow:
      users: []
      groups: ["-200100"]
    admin:
      users: []
    slash:
      global:
        users: []
  groups:
    "-200100":
      admin_users:
        users: ["111"]
"#,
    );

    assert!(!channel.is_authorized_for_slash_command_for_recipient(
        "111",
        "session.status",
        "-200100",
    ));
}
