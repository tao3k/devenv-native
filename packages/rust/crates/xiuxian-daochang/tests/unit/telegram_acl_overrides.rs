#![allow(
    missing_docs,
    unused_imports,
    dead_code,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::field_reassign_with_default,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::map_unwrap_or,
    clippy::option_as_ref_deref,
    clippy::unreadable_literal,
    clippy::useless_conversion,
    clippy::match_wildcard_for_single_variants,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_raw_string_hashes,
    clippy::manual_async_fn,
    clippy::manual_let_else,
    clippy::too_many_lines,
    clippy::unnecessary_literal_bound,
    clippy::needless_pass_by_value,
    clippy::struct_field_names,
    clippy::single_match_else,
    clippy::assigning_clones
)]

use std::path::PathBuf;

use tempfile::TempDir;
use xiuxian_daochang::{
    Channel, TelegramChannel, TelegramControlCommandPolicy, TelegramSessionPartition,
    build_telegram_acl_overrides, load_runtime_settings_from_paths,
};

fn write_file(path: PathBuf, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent dir");
    }
    std::fs::write(path, content).expect("write toml");
}

#[test]
fn telegram_acl_overrides_build_from_structured_acl() {
    let tmp = TempDir::new().expect("tempdir");
    let system = tmp
        .path()
        .join("packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml");
    let user = tmp
        .path()
        .join(".config/xiuxian-artisan-workshop/xiuxian.toml");

    write_file(
        system.clone(),
        r#"
[telegram.acl.allow]
users = ["1001", "1002"]
groups = ["-2001", "*"]

[telegram.acl.admin]
users = ["1001"]

[telegram.acl.control.allow_from]
users = ["1001", "1003"]

[[telegram.acl.control.rules]]
commands = ["/session partition"]

[telegram.acl.control.rules.allow]
users = ["1001"]

[[telegram.acl.control.rules]]
commands = ["/reset", "/clear"]

[telegram.acl.control.rules.allow]
users = ["2001"]

[telegram.acl.slash.global]
users = ["1001", "1003"]

[telegram.acl.slash.session_status]
users = ["1004"]

[telegram.acl.slash.session_budget]
users = ["1004"]

[telegram.acl.slash.session_memory]
users = ["1005"]

[telegram.acl.slash.session_feedback]
users = ["1005"]

[telegram.acl.slash.job_status]
users = ["1006"]

[telegram.acl.slash.jobs_summary]
users = ["1006"]

[telegram.acl.slash.background_submit]
users = ["1006"]
"#,
    );
    write_file(user.clone(), "");

    let settings = load_runtime_settings_from_paths(&system, &user);
    let overrides = build_telegram_acl_overrides(&settings).expect("telegram acl overrides");

    assert_eq!(overrides.allowed_users, vec!["1001", "1002"]);
    assert_eq!(overrides.allowed_groups, vec!["-2001", "*"]);
    assert_eq!(overrides.admin_users, vec!["1001"]);
    assert_eq!(
        overrides.control_command_allow_from,
        Some(vec!["1001".to_string(), "1003".to_string()])
    );
    assert_eq!(overrides.control_command_rules.len(), 2);
    let channel = TelegramChannel::new_with_partition_and_control_command_policy(
        "fake-token".to_string(),
        vec!["*".to_string()],
        vec![],
        TelegramControlCommandPolicy::new(
            vec!["9001".to_string()],
            None,
            overrides.control_command_rules.clone(),
        ),
        TelegramSessionPartition::ChatUser,
    )
    .expect("typed control rules should compile");
    assert!(channel.is_authorized_for_control_command("1001", "/session partition"));
    assert!(channel.is_authorized_for_control_command("2001", "/reset"));
    assert!(
        !channel.is_authorized_for_control_command("9001", "/session partition"),
        "matched command-scoped rule should override admin fallback",
    );
    assert_eq!(
        overrides.slash_command_allow_from,
        Some(vec!["1001".to_string(), "1003".to_string()])
    );
    assert_eq!(
        overrides.slash_session_status_allow_from,
        Some(vec!["1004".to_string()])
    );
    assert_eq!(
        overrides.slash_session_budget_allow_from,
        Some(vec!["1004".to_string()])
    );
    assert_eq!(
        overrides.slash_session_memory_allow_from,
        Some(vec!["1005".to_string()])
    );
    assert_eq!(
        overrides.slash_session_feedback_allow_from,
        Some(vec!["1005".to_string()])
    );
    assert_eq!(
        overrides.slash_job_allow_from,
        Some(vec!["1006".to_string()])
    );
    assert_eq!(
        overrides.slash_jobs_allow_from,
        Some(vec!["1006".to_string()])
    );
    assert_eq!(
        overrides.slash_bg_allow_from,
        Some(vec!["1006".to_string()])
    );
}

#[test]
fn telegram_acl_overrides_use_user_settings_for_acl_merge() {
    let tmp = TempDir::new().expect("tempdir");
    let system = tmp
        .path()
        .join("packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml");
    let user = tmp
        .path()
        .join(".config/xiuxian-artisan-workshop/xiuxian.toml");

    write_file(
        system.clone(),
        r#"
[telegram.acl.allow]
users = ["1001"]
groups = ["-2001"]

[telegram.acl.admin]
users = ["1001"]
"#,
    );
    write_file(
        user.clone(),
        r#"
[telegram.acl.allow]
users = ["2002"]

[telegram.acl.admin]
users = ["2002"]
"#,
    );

    let settings = load_runtime_settings_from_paths(&system, &user);
    let overrides = build_telegram_acl_overrides(&settings).expect("telegram acl overrides");

    assert_eq!(overrides.allowed_users, vec!["2002"]);
    assert_eq!(
        overrides.allowed_groups,
        vec!["-2001"],
        "group allowlist should inherit from system when user override omits groups"
    );
    assert_eq!(overrides.admin_users, vec!["2002"]);
}

#[test]
fn telegram_acl_overrides_reject_invalid_control_rule_with_field_path() {
    let tmp = TempDir::new().expect("tempdir");
    let system = tmp
        .path()
        .join("packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml");
    let user = tmp
        .path()
        .join(".config/xiuxian-artisan-workshop/xiuxian.toml");

    write_file(
        system.clone(),
        r#"
[[telegram.acl.control.rules]]
commands = ["session*"]

[telegram.acl.control.rules.allow]
users = ["1001"]
"#,
    );
    write_file(user, "");

    let settings = load_runtime_settings_from_paths(
        &system,
        &tmp.path()
            .join(".config/xiuxian-artisan-workshop/xiuxian.toml"),
    );
    let error = build_telegram_acl_overrides(&settings)
        .expect_err("invalid command selector should fail fast");
    assert!(
        error
            .to_string()
            .contains("telegram.acl.control.rules[0].commands"),
        "unexpected error path: {error}",
    );
}
