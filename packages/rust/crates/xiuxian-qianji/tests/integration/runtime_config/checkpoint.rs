use super::*;

#[test]
fn runtime_checkpoint_config_uses_system_defaults() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml"),
        r#"
[checkpoint]
valkey_url = "redis://system.example.com:6379/2"
"#,
    );

    let cfg = resolve_checkpoint(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.valkey_url, "redis://system.example.com:6379/2");
}

#[test]
fn runtime_checkpoint_config_prefers_qianji_toml_over_env_fallbacks() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml"),
        r#"
[checkpoint]
valkey_url = "redis://system.example.com:6379/2"
"#,
    );
    write_file(
        &config_home.join("xiuxian-artisan-workshop/qianji.toml"),
        r#"
[checkpoint]
valkey_url = "redis://user.example.com:6379/3"
"#,
    );

    let cfg = resolve_checkpoint(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        extra_env: vec![
            (
                "QIANJI_VALKEY_URL".to_string(),
                "redis://env.example.com:6379/4".to_string(),
            ),
            (
                "VALKEY_URL".to_string(),
                "redis://legacy-env.example.com:6379/5".to_string(),
            ),
        ],
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.valkey_url, "redis://user.example.com:6379/3");
}

#[test]
fn runtime_checkpoint_config_uses_env_fallback_when_toml_missing() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    let cfg = resolve_checkpoint(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        extra_env: vec![(
            "QIANJI_VALKEY_URL".to_string(),
            "redis://env.example.com:6379/4".to_string(),
        )],
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.valkey_url, "redis://env.example.com:6379/4");
}
