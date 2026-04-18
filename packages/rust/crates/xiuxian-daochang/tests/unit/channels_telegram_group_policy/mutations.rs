use super::support::{
    Channel, RecipientCommandAdminUsersMutation, build_channel_with_settings,
    load_runtime_settings_from_paths, settings_paths,
};

#[test]
fn telegram_group_policy_recipient_admin_users_runtime_mutation_group_scope() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
telegram:
  acl:
    allow:
      users: []
      groups: ["-200100"]
    admin:
      users: []
"#,
    );

    assert_eq!(
        channel
            .recipient_command_admin_users("-200100")
            .expect("group recipient query should succeed"),
        None
    );

    assert_eq!(
        channel
            .mutate_recipient_command_admin_users(
                "-200100",
                RecipientCommandAdminUsersMutation::Add(vec!["telegram:111".to_string()]),
            )
            .expect("group add should succeed"),
        Some(vec!["111".to_string()])
    );
    assert!(channel.is_authorized_for_control_command_for_recipient(
        "111",
        "/session partition",
        "-200100",
    ));

    assert_eq!(
        channel
            .mutate_recipient_command_admin_users(
                "-200100",
                RecipientCommandAdminUsersMutation::Remove(vec!["111".to_string()]),
            )
            .expect("group remove should succeed"),
        None
    );
    assert_eq!(
        channel
            .recipient_command_admin_users("-200100")
            .expect("group recipient query should succeed"),
        None
    );
}

#[test]
fn telegram_group_policy_recipient_admin_users_runtime_mutation_topic_scope() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
telegram:
  acl:
    allow:
      users: []
      groups: ["-200100"]
    admin:
      users: []
"#,
    );

    assert_eq!(
        channel
            .mutate_recipient_command_admin_users(
                "-200100:42",
                RecipientCommandAdminUsersMutation::Set(vec!["222".to_string(), "222".to_string()]),
            )
            .expect("topic set should succeed"),
        Some(vec!["222".to_string()])
    );
    assert!(channel.is_authorized_for_control_command_for_recipient(
        "222",
        "/session admin",
        "-200100:42",
    ));
    assert!(!channel.is_authorized_for_control_command_for_recipient(
        "222",
        "/session admin",
        "-200100:99",
    ));

    assert_eq!(
        channel
            .mutate_recipient_command_admin_users(
                "-200100:42",
                RecipientCommandAdminUsersMutation::Clear,
            )
            .expect("topic clear should succeed"),
        None
    );
    assert_eq!(
        channel
            .recipient_command_admin_users("-200100:42")
            .expect("topic query should succeed"),
        None
    );
}

#[test]
fn telegram_group_policy_recipient_admin_users_runtime_mutation_group_topic_isolation() {
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
telegram:
  acl:
    allow:
      users: []
      groups: ["-200100"]
    admin:
      users: []
"#,
    );

    assert_eq!(
        channel
            .mutate_recipient_command_admin_users(
                "-200100",
                RecipientCommandAdminUsersMutation::Set(vec!["111".to_string()]),
            )
            .expect("group set should succeed"),
        Some(vec!["111".to_string()])
    );
    assert_eq!(
        channel
            .mutate_recipient_command_admin_users(
                "-200100:42",
                RecipientCommandAdminUsersMutation::Set(vec!["222".to_string()]),
            )
            .expect("topic set should succeed"),
        Some(vec!["222".to_string()])
    );

    assert!(!channel.is_authorized_for_control_command_for_recipient(
        "111",
        "/session admin",
        "-200100:42",
    ));
    assert!(channel.is_authorized_for_control_command_for_recipient(
        "111",
        "/session admin",
        "-200100:99",
    ));
    assert!(channel.is_authorized_for_control_command_for_recipient(
        "222",
        "/session admin",
        "-200100:42",
    ));
    assert!(!channel.is_authorized_for_control_command_for_recipient(
        "222",
        "/session admin",
        "-200100:99",
    ));
}

#[test]
fn telegram_group_policy_recipient_admin_users_runtime_mutation_rejects_invalid_identity_or_scope()
{
    let (_temp_dir, channel) = build_channel_with_settings(
        r#"
telegram:
  acl:
    allow:
      users: []
      groups: ["-200100"]
    admin:
      users: []
"#,
    );

    assert!(
        channel
            .mutate_recipient_command_admin_users(
                "-200100",
                RecipientCommandAdminUsersMutation::Set(vec!["alice".to_string()]),
            )
            .is_err(),
        "set should reject non-numeric identity"
    );
    assert!(
        channel.recipient_command_admin_users("12345").is_err(),
        "direct-chat recipient should not support delegated admin mutation"
    );
}

#[test]
fn telegram_group_policy_recipient_admin_users_runtime_mutation_persists_when_enabled() {
    let (temp_dir, channel) = build_channel_with_settings(
        r#"
telegram:
  acl:
    allow:
      users: []
      groups: ["-200100"]
    admin:
      users: []
  session_admin_persist: true
"#,
    );

    assert_eq!(
        channel
            .mutate_recipient_command_admin_users(
                "-200100",
                RecipientCommandAdminUsersMutation::Add(vec!["telegram:111".to_string()]),
            )
            .expect("group add should succeed"),
        Some(vec!["111".to_string()])
    );
    assert_eq!(
        channel
            .mutate_recipient_command_admin_users(
                "-200100:42",
                RecipientCommandAdminUsersMutation::Set(vec![
                    "222".to_string(),
                    "telegram:333".to_string(),
                    "222".to_string(),
                ]),
            )
            .expect("topic set should succeed"),
        Some(vec!["222".to_string(), "333".to_string()])
    );

    let (system_settings_path, user_settings_path) = settings_paths(&temp_dir);
    let merged = load_runtime_settings_from_paths(&system_settings_path, &user_settings_path);
    let groups = merged
        .telegram
        .groups
        .expect("group overrides should persist");
    let group = groups.get("-200100").expect("group override should exist");
    assert_eq!(
        group
            .admin_users
            .as_ref()
            .and_then(|value| value.users.clone()),
        Some(vec!["111".to_string()])
    );
    let topics = group
        .topics
        .as_ref()
        .expect("topic override should persist");
    let topic = topics.get("42").expect("topic override should exist");
    assert_eq!(
        topic
            .admin_users
            .as_ref()
            .and_then(|value| value.users.clone()),
        Some(vec!["222".to_string(), "333".to_string()])
    );
}

#[test]
fn telegram_group_policy_recipient_admin_users_runtime_mutation_clear_prunes_persisted_entries() {
    let (temp_dir, channel) = build_channel_with_settings(
        r#"
telegram:
  acl:
    allow:
      users: []
      groups: ["-200100"]
    admin:
      users: []
  session_admin_persist: true
"#,
    );

    channel
        .mutate_recipient_command_admin_users(
            "-200100",
            RecipientCommandAdminUsersMutation::Set(vec!["111".to_string()]),
        )
        .expect("group set should succeed");
    channel
        .mutate_recipient_command_admin_users(
            "-200100:42",
            RecipientCommandAdminUsersMutation::Set(vec!["222".to_string()]),
        )
        .expect("topic set should succeed");
    channel
        .mutate_recipient_command_admin_users(
            "-200100:42",
            RecipientCommandAdminUsersMutation::Clear,
        )
        .expect("topic clear should succeed");
    channel
        .mutate_recipient_command_admin_users("-200100", RecipientCommandAdminUsersMutation::Clear)
        .expect("group clear should succeed");

    let (system_settings_path, user_settings_path) = settings_paths(&temp_dir);
    let merged = load_runtime_settings_from_paths(&system_settings_path, &user_settings_path);
    assert!(
        merged.telegram.groups.is_none(),
        "clearing overrides should prune persisted group/topic admin entries"
    );
    let user_yaml =
        std::fs::read_to_string(user_settings_path).expect("user settings should be readable");
    assert!(!user_yaml.contains("admin_users"));
    assert!(!user_yaml.contains("-200100"));
    assert!(!user_yaml.contains("42"));
}

#[test]
fn telegram_group_policy_recipient_admin_users_runtime_mutation_does_not_persist_when_disabled() {
    let (temp_dir, channel) = build_channel_with_settings(
        r#"
telegram:
  acl:
    allow:
      users: []
      groups: ["-200100"]
    admin:
      users: []
  session_admin_persist: false
"#,
    );

    assert_eq!(
        channel
            .mutate_recipient_command_admin_users(
                "-200100",
                RecipientCommandAdminUsersMutation::Set(vec!["111".to_string()]),
            )
            .expect("group set should succeed"),
        Some(vec!["111".to_string()])
    );
    assert_eq!(
        channel
            .recipient_command_admin_users("-200100")
            .expect("group query should succeed"),
        Some(vec!["111".to_string()])
    );

    let (system_settings_path, user_settings_path) = settings_paths(&temp_dir);
    let user_yaml =
        std::fs::read_to_string(&user_settings_path).expect("user settings should be readable");
    assert!(
        user_yaml.trim().is_empty(),
        "persistence-disabled mode must not mutate user settings"
    );

    channel.reload_acl_from_settings_for_test();
    assert_eq!(
        channel
            .recipient_command_admin_users("-200100")
            .expect("group query should succeed"),
        None,
        "process-local override should disappear after reload when persistence is disabled"
    );

    let merged = load_runtime_settings_from_paths(&system_settings_path, &user_settings_path);
    assert!(merged.telegram.groups.is_none());
}
