use std::fs;

pub(super) use xiuxian_daochang::{
    Channel, RecipientCommandAdminUsersMutation, load_runtime_settings_from_paths,
};
use xiuxian_daochang::{TelegramChannel, TelegramControlCommandPolicy, TelegramSessionPartition};

pub(super) fn build_channel_with_settings(
    settings_yaml: &str,
) -> (tempfile::TempDir, TelegramChannel) {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let system_settings_path = temp_dir.path().join("settings-system.yaml");
    let user_settings_path = temp_dir.path().join("settings-user.yaml");
    let settings_toml = yaml_to_toml(settings_yaml);
    fs::write(&system_settings_path, settings_toml).expect("write system settings");
    fs::write(&user_settings_path, "").expect("write user settings");

    let channel = TelegramChannel::new_with_partition_and_control_command_policy(
        "fake-token".to_string(),
        vec![],
        vec![],
        TelegramControlCommandPolicy::default(),
        TelegramSessionPartition::ChatUser,
    );
    channel.set_acl_reload_paths_for_test(system_settings_path, user_settings_path);
    channel.reload_acl_from_settings_for_test();
    (temp_dir, channel)
}

fn yaml_to_toml(value: &str) -> String {
    let yaml: serde_yaml::Value = serde_yaml::from_str(value).expect("parse yaml fixture");
    toml::to_string(&yaml).expect("convert yaml fixture to toml")
}

pub(super) fn settings_paths(
    temp_dir: &tempfile::TempDir,
) -> (std::path::PathBuf, std::path::PathBuf) {
    (
        temp_dir.path().join("settings-system.yaml"),
        temp_dir.path().join("settings-user.yaml"),
    )
}

pub(super) fn group_update_for_chat(chat_id: i64, user_id: i64, text: &str) -> serde_json::Value {
    serde_json::json!({
        "update_id": 90001,
        "message": {
            "message_id": 101,
            "text": text,
            "chat": { "id": chat_id, "type": "group", "title": "team" },
            "from": { "id": user_id, "username": format!("u{user_id}") }
        }
    })
}

pub(super) fn group_update(user_id: i64, text: &str) -> serde_json::Value {
    group_update_for_chat(-200100, user_id, text)
}

pub(super) fn topic_update(user_id: i64, text: &str, topic_id: i64) -> serde_json::Value {
    let mut update = group_update(user_id, text);
    update["message"]["message_thread_id"] = serde_json::json!(topic_id);
    update
}
