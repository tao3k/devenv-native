use super::*;

#[test]
fn runtime_config_uses_system_file_defaults() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml"),
        r#"
[llm]
model = "system-model"
base_url = "http://system.local/v1"
api_key_env = "SYSTEM_API_KEY"
"#,
    );

    let cfg = resolve(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        extra_env: vec![
            ("QIANJI_LLM_MODEL".to_string(), String::new()),
            ("OPENAI_API_BASE".to_string(), String::new()),
            ("OPENAI_API_KEY".to_string(), String::new()),
            ("SYSTEM_API_KEY".to_string(), "system-secret".to_string()),
        ],
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.model, "system-model");
    assert_eq!(cfg.base_url, "http://system.local/v1");
    assert_eq!(cfg.api_key_env, "SYSTEM_API_KEY");
    assert_eq!(cfg.wire_api, "chat_completions");
    assert_eq!(cfg.api_key, "system-secret");
}

#[test]
fn runtime_config_user_file_overrides_system_file() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml"),
        r#"
[llm]
model = "system-model"
base_url = "http://system.local/v1"
api_key_env = "SYSTEM_API_KEY"
"#,
    );
    write_file(
        &config_home.join("xiuxian-artisan-workshop/qianji.toml"),
        r#"
[llm]
model = "user-model"
base_url = "http://user.local/v1"
api_key_env = "USER_API_KEY"
"#,
    );

    let cfg = resolve(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        extra_env: vec![
            ("QIANJI_LLM_MODEL".to_string(), String::new()),
            ("OPENAI_API_BASE".to_string(), String::new()),
            ("OPENAI_API_KEY".to_string(), String::new()),
            ("USER_API_KEY".to_string(), "user-secret".to_string()),
        ],
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.model, "user-model");
    assert_eq!(cfg.base_url, "http://user.local/v1");
    assert_eq!(cfg.api_key_env, "USER_API_KEY");
    assert_eq!(cfg.wire_api, "chat_completions");
    assert_eq!(cfg.api_key, "user-secret");
}

#[test]
fn runtime_config_explicit_file_supports_imports() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");
    let explicit_config = project_root.join("custom-qianji.toml");
    let shared_config = project_root.join("qianji.shared.toml");

    write_file(
        &shared_config,
        r#"
[llm]
base_url = "http://shared.local/v1"
api_key_env = "SHARED_API_KEY"
"#,
    );
    write_file(
        &explicit_config,
        r#"
imports = ["qianji.shared.toml"]

[llm]
model = "imported-model"
"#,
    );

    let cfg = resolve(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        qianji_config_path: Some(explicit_config),
        extra_env: vec![
            ("QIANJI_LLM_MODEL".to_string(), String::new()),
            ("OPENAI_API_BASE".to_string(), String::new()),
            ("OPENAI_API_KEY".to_string(), String::new()),
            ("SHARED_API_KEY".to_string(), "shared-secret".to_string()),
        ],
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.model, "imported-model");
    assert_eq!(cfg.base_url, "http://shared.local/v1");
    assert_eq!(cfg.api_key_env, "SHARED_API_KEY");
    assert_eq!(cfg.api_key, "shared-secret");
}

#[test]
fn runtime_config_explicit_path_overrides_user_and_system() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");
    let explicit_path = tmp.path().join("explicit/qianji.toml");

    write_file(
        &project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml"),
        r#"
[llm]
model = "system-model"
base_url = "http://system.local/v1"
api_key_env = "SYSTEM_API_KEY"
"#,
    );
    write_file(
        &config_home.join("xiuxian-artisan-workshop/qianji.toml"),
        r#"
[llm]
model = "user-model"
base_url = "http://user.local/v1"
api_key_env = "USER_API_KEY"
"#,
    );
    write_file(
        &explicit_path,
        r#"
[llm]
model = "explicit-model"
base_url = "http://explicit.local/v1"
api_key_env = "EXPLICIT_API_KEY"
"#,
    );

    let cfg = resolve(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        qianji_config_path: Some(explicit_path),
        extra_env: vec![
            ("QIANJI_LLM_MODEL".to_string(), String::new()),
            ("OPENAI_API_BASE".to_string(), String::new()),
            ("OPENAI_API_KEY".to_string(), String::new()),
            (
                "EXPLICIT_API_KEY".to_string(),
                "explicit-secret".to_string(),
            ),
        ],
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.model, "explicit-model");
    assert_eq!(cfg.base_url, "http://explicit.local/v1");
    assert_eq!(cfg.api_key_env, "EXPLICIT_API_KEY");
    assert_eq!(cfg.wire_api, "chat_completions");
    assert_eq!(cfg.api_key, "explicit-secret");
}

#[test]
fn runtime_config_uses_user_xiuxian_toml_as_llm_overlay_when_user_qianji_toml_is_missing() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml"),
        r#"
[llm]
model = "system-model"
base_url = "http://system.local/v1"
api_key_env = "SYSTEM_API_KEY"
"#,
    );
    write_file(
        &config_home.join("xiuxian-artisan-workshop/xiuxian.toml"),
        r#"
[llm]
default_model = "legacy-user-model"
default_provider = "openai"

[llm.providers.openai]
base_url = "http://legacy-user.local/v1"
api_key = "LEGACY_USER_API_KEY"
model = "legacy-user-model"
"#,
    );

    let cfg = resolve(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        extra_env: vec![
            (
                "OPENAI_API_KEY".to_string(),
                "generic-openai-secret".to_string(),
            ),
            (
                "LEGACY_USER_API_KEY".to_string(),
                "legacy-user-secret".to_string(),
            ),
        ],
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.model, "legacy-user-model");
    assert_eq!(cfg.base_url, "http://legacy-user.local/v1");
    assert_eq!(cfg.api_key_env, "LEGACY_USER_API_KEY");
    assert_eq!(cfg.api_key, "generic-openai-secret");
}

#[test]
fn runtime_config_user_qianji_toml_overrides_user_xiuxian_toml() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml"),
        r#"
[llm]
model = "system-model"
base_url = "http://system.local/v1"
api_key_env = "SYSTEM_API_KEY"
"#,
    );
    write_file(
        &config_home.join("xiuxian-artisan-workshop/xiuxian.toml"),
        r#"
[llm]
default_model = "legacy-user-model"
default_provider = "openai"

[llm.providers.openai]
base_url = "http://legacy-user.local/v1"
api_key = "LEGACY_USER_API_KEY"
model = "legacy-user-model"
"#,
    );
    write_file(
        &config_home.join("xiuxian-artisan-workshop/qianji.toml"),
        r#"
[llm]
model = "user-qianji-model"
base_url = "http://user-qianji.local/v1"
api_key_env = "USER_QIANJI_API_KEY"
"#,
    );

    let cfg = resolve(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        extra_env: vec![
            ("OPENAI_API_KEY".to_string(), String::new()),
            (
                "LEGACY_USER_API_KEY".to_string(),
                "legacy-user-secret".to_string(),
            ),
            (
                "USER_QIANJI_API_KEY".to_string(),
                "user-qianji-secret".to_string(),
            ),
        ],
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.model, "user-qianji-model");
    assert_eq!(cfg.base_url, "http://user-qianji.local/v1");
    assert_eq!(cfg.api_key_env, "USER_QIANJI_API_KEY");
    assert_eq!(cfg.api_key, "user-qianji-secret");
}

#[test]
fn runtime_config_env_overrides_file_layers() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml"),
        r#"
[llm]
model = "system-model"
base_url = "http://system.local/v1"
api_key_env = "SYSTEM_API_KEY"
"#,
    );

    let cfg = resolve(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        qianji_llm_model: Some("env-model".to_string()),
        openai_api_base: Some("http://env.local/v1".to_string()),
        openai_api_key: Some("env-openai-key".to_string()),
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.model, "env-model");
    assert_eq!(cfg.base_url, "http://env.local/v1");
    assert_eq!(cfg.api_key_env, "SYSTEM_API_KEY");
    assert_eq!(cfg.wire_api, "chat_completions");
    assert_eq!(cfg.api_key, "env-openai-key");
}

#[test]
fn runtime_config_prefers_openai_api_key_over_named_env_key() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml"),
        r#"
[llm]
model = "system-model"
base_url = "http://system.local/v1"
api_key_env = "SYSTEM_API_KEY"
"#,
    );

    let cfg = resolve(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        extra_env: vec![
            ("OPENAI_API_KEY".to_string(), "openai-secret".to_string()),
            ("SYSTEM_API_KEY".to_string(), "system-secret".to_string()),
        ],
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.api_key_env, "SYSTEM_API_KEY");
    assert_eq!(cfg.wire_api, "chat_completions");
    assert_eq!(cfg.api_key, "openai-secret");
}

#[test]
fn runtime_config_parse_error_surfaces_as_invalid_data() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");
    write_file(
        &project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml"),
        "this is not valid toml = ]",
    );

    let result = resolve_qianji_runtime_llm_config_with_env(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        ..QianjiRuntimeEnv::default()
    });

    let Err(err) = result else {
        panic!("invalid qianji.toml should return error");
    };
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
}

#[test]
fn runtime_config_missing_api_key_returns_not_found() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml"),
        r#"
[llm]
model = "system-model"
base_url = "http://system.local/v1"
api_key_env = "SYSTEM_API_KEY"
"#,
    );

    let result = resolve_qianji_runtime_llm_config_with_env(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        extra_env: vec![
            ("OPENAI_API_KEY".to_string(), String::new()),
            ("SYSTEM_API_KEY".to_string(), String::new()),
        ],
        ..QianjiRuntimeEnv::default()
    });

    let Err(err) = result else {
        panic!("missing API key should return error");
    };
    assert_eq!(err.kind(), io::ErrorKind::NotFound);
    assert!(err.to_string().contains("SYSTEM_API_KEY"));
}

#[cfg(feature = "llm")]
#[test]
fn runtime_config_resolves_default_provider_wire_api_from_qianji_toml() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml"),
        r#"
[llm]
default_provider = "openai"
default_model = "fallback-model"
"#,
    );
    write_file(
        &config_home.join("xiuxian-artisan-workshop/qianji.toml"),
        r#"
[llm]
default_provider = "openai"

[llm.providers.openai]
model = "gpt-5-codex"
base_url = "https://openai-compatible.example.com/v1"
api_key = "OPENAI_API_KEY"
wire_api = "responses"
"#,
    );

    let cfg = resolve(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        extra_env: vec![("OPENAI_API_KEY".to_string(), "test-openai-key".to_string())],
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.model, "gpt-5-codex");
    assert_eq!(cfg.base_url, "https://openai-compatible.example.com/v1");
    assert_eq!(cfg.api_key_env, "OPENAI_API_KEY");
    assert_eq!(cfg.wire_api, "responses");
    assert_eq!(cfg.api_key, "test-openai-key");
}
