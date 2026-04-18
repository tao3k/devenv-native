use super::support::{
    build_telegram_acl_overrides, load_runtime_settings_from_paths, new_temp_settings_paths,
    require_ok, write_file,
};

#[test]
fn missing_files_fallback_to_defaults() {
    let (tmp, _system, _user) = new_temp_settings_paths();
    let merged = load_runtime_settings_from_paths(
        &tmp.path().join("missing-system.toml"),
        &tmp.path().join("missing-user.toml"),
    );
    let telegram_overrides = require_ok(
        build_telegram_acl_overrides(&merged),
        "telegram acl overrides",
    );
    assert!(telegram_overrides.allowed_users.is_empty());
    assert!(telegram_overrides.allowed_groups.is_empty());
    assert!(merged.telegram.group_policy.is_none());
    assert!(merged.tool_runtime.pool_size.is_none());
    assert!(merged.embedding.backend.is_none());
    assert!(merged.memory.embedding_timeout_ms.is_none());
}

#[test]
fn invalid_toml_is_ignored() {
    let (tmp, system, user) = new_temp_settings_paths();

    write_file(&system, "[telegram");
    write_file(
        user,
        r#"
[telegram.acl.allow]
users = ["1001"]
"#,
    );

    let merged = load_runtime_settings_from_paths(
        &tmp.path()
            .join("packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml"),
        &tmp.path()
            .join(".config/xiuxian-artisan-workshop/xiuxian.toml"),
    );
    let telegram_overrides = require_ok(
        build_telegram_acl_overrides(&merged),
        "telegram acl overrides",
    );
    assert_eq!(telegram_overrides.allowed_users, vec!["1001"]);
}
