use super::support::{load_runtime_settings_from_paths, new_temp_settings_paths, write_file};

#[test]
fn merge_channel_foreground_queue_mode_uses_user_override() {
    let (_tmp, system, user) = new_temp_settings_paths();

    write_file(
        &system,
        r#"
[telegram]
foreground_queue_mode = "interrupt"

[discord]
foreground_queue_mode = "interrupt"
"#,
    );
    write_file(
        &user,
        r#"
[telegram]
foreground_queue_mode = "queue"

[discord]
foreground_queue_mode = "queue"
"#,
    );

    let merged = load_runtime_settings_from_paths(&system, &user);
    assert_eq!(
        merged.telegram.foreground_queue_mode.as_deref(),
        Some("queue")
    );
    assert_eq!(
        merged.discord.foreground_queue_mode.as_deref(),
        Some("queue")
    );
}
