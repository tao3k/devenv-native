use super::support::{load_runtime_settings_from_paths, new_temp_settings_paths, write_file};

#[test]
fn merge_discord_session_partition_persist_uses_user_override() {
    let (_tmp, system, user) = new_temp_settings_paths();

    write_file(
        &system,
        r"
[discord]
session_partition_persist = true
",
    );
    write_file(
        &user,
        r"
[discord]
session_partition_persist = false
",
    );

    let merged = load_runtime_settings_from_paths(&system, &user);
    assert_eq!(merged.discord.session_partition_persist, Some(false));
}

#[test]
fn merge_discord_mention_policy_overrides_deeply() {
    let (_tmp, system, user) = new_temp_settings_paths();

    write_file(
        &system,
        r#"
[discord]
require_mention = true
require_mention_persist = false

[discord.channels."*"]
require_mention = true

[discord.channels."2001"]
require_mention = false
"#,
    );
    write_file(
        &user,
        r#"
[discord]
require_mention = false
require_mention_persist = true

[discord.channels."2001"]
require_mention = true

[discord.channels."2002"]
require_mention = false
"#,
    );

    let merged = load_runtime_settings_from_paths(&system, &user);
    assert_eq!(merged.discord.require_mention, Some(false));
    assert_eq!(merged.discord.require_mention_persist, Some(true));
    let Some(channels) = merged.discord.channels.as_ref() else {
        panic!("discord channels");
    };
    assert_eq!(
        channels.get("*").and_then(|value| value.require_mention),
        Some(true)
    );
    assert_eq!(
        channels.get("2001").and_then(|value| value.require_mention),
        Some(true)
    );
    assert_eq!(
        channels.get("2002").and_then(|value| value.require_mention),
        Some(false)
    );
}
