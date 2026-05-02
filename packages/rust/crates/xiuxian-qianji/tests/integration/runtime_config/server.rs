use super::{resolve_server, write_file};
use tempfile::TempDir;
use xiuxian_qianji::runtime_config::QianjiRuntimeEnv;

#[test]
fn runtime_server_config_uses_system_defaults() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml"),
        r#"
[server]
bind_addr = "127.0.0.1:38131"
"#,
    );

    let cfg = resolve_server(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.bind_addr, "127.0.0.1:38131");
    assert!(!cfg.require_valkey_ready);
}

#[test]
fn runtime_server_config_user_file_overrides_system_file() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml"),
        r#"
[server]
bind_addr = "127.0.0.1:38131"
"#,
    );
    write_file(
        &config_home.join("xiuxian-artisan-workshop/qianji.toml"),
        r#"
[server]
bind_addr = "127.0.0.1:38132"
"#,
    );

    let cfg = resolve_server(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.bind_addr, "127.0.0.1:38132");
    assert!(!cfg.require_valkey_ready);
}

#[test]
fn runtime_server_config_prefers_qianji_toml_over_env_fallback() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml"),
        r#"
[server]
bind_addr = "127.0.0.1:38131"
"#,
    );

    let cfg = resolve_server(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        extra_env: vec![(
            "QIANJI_SERVER_BIND_ADDR".to_string(),
            "127.0.0.1:38133".to_string(),
        )],
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.bind_addr, "127.0.0.1:38131");
    assert!(!cfg.require_valkey_ready);
}

#[test]
fn runtime_server_config_uses_env_fallback_when_toml_missing() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    let cfg = resolve_server(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        extra_env: vec![(
            "QIANJI_SERVER_BIND_ADDR".to_string(),
            "127.0.0.1:38134".to_string(),
        )],
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.bind_addr, "127.0.0.1:38134");
    assert!(!cfg.require_valkey_ready);
}

#[test]
fn runtime_server_config_runtime_env_overrides_file() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml"),
        r#"
[server]
bind_addr = "127.0.0.1:38131"
"#,
    );

    let cfg = resolve_server(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        qianji_server_bind_addr: Some("127.0.0.1:38135".to_string()),
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.bind_addr, "127.0.0.1:38135");
    assert!(!cfg.require_valkey_ready);
}

#[test]
fn runtime_server_config_reads_require_valkey_ready_from_qianji_toml() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml"),
        r#"
[server]
bind_addr = "127.0.0.1:38131"
require_valkey_ready = true
"#,
    );

    let cfg = resolve_server(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.bind_addr, "127.0.0.1:38131");
    assert!(cfg.require_valkey_ready);
}

#[test]
fn runtime_server_config_uses_require_valkey_ready_env_fallback_when_toml_missing() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    let cfg = resolve_server(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        extra_env: vec![(
            "QIANJI_SERVER_REQUIRE_VALKEY_READY".to_string(),
            "true".to_string(),
        )],
        ..QianjiRuntimeEnv::default()
    });

    assert!(cfg.require_valkey_ready);
}

#[test]
fn runtime_server_config_runtime_env_overrides_require_valkey_ready_file() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml"),
        r"
[server]
require_valkey_ready = true
",
    );

    let cfg = resolve_server(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        qianji_server_require_valkey_ready: Some(false),
        ..QianjiRuntimeEnv::default()
    });

    assert!(!cfg.require_valkey_ready);
}
