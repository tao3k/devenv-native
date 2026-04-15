use super::*;

#[test]
fn runtime_wendao_config_uses_system_defaults() {
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

[memory_promotion.wendao]
graph_scope = "scope:system"
graph_scope_key = "promotion_scope"
graph_dimension = 2048
persist = true
persist_best_effort = false
"#,
    );

    let cfg = resolve_wendao(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.graph_scope, "scope:system");
    assert_eq!(cfg.graph_scope_key.as_deref(), Some("promotion_scope"));
    assert_eq!(cfg.graph_dimension, 2048);
    assert!(cfg.persist);
    assert!(!cfg.persist_best_effort);
}

#[test]
fn runtime_wendao_config_env_overrides_file() {
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

[memory_promotion.wendao]
graph_scope = "scope:system"
graph_dimension = 2048
persist = true
persist_best_effort = true
"#,
    );

    let cfg = resolve_wendao(&QianjiRuntimeEnv {
        prj_root: Some(project_root),
        prj_config_home: Some(config_home),
        extra_env: vec![
            (
                "QIANJI_MEMORY_PROMOTION_GRAPH_SCOPE".to_string(),
                "scope:env".to_string(),
            ),
            (
                "QIANJI_MEMORY_PROMOTION_GRAPH_SCOPE_KEY".to_string(),
                "scope_key_env".to_string(),
            ),
            (
                "QIANJI_MEMORY_PROMOTION_GRAPH_DIMENSION".to_string(),
                "4096".to_string(),
            ),
            (
                "QIANJI_MEMORY_PROMOTION_PERSIST".to_string(),
                "false".to_string(),
            ),
            (
                "QIANJI_MEMORY_PROMOTION_PERSIST_BEST_EFFORT".to_string(),
                "false".to_string(),
            ),
        ],
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(cfg.graph_scope, "scope:env");
    assert_eq!(cfg.graph_scope_key.as_deref(), Some("scope_key_env"));
    assert_eq!(cfg.graph_dimension, 4096);
    assert!(!cfg.persist);
    assert!(!cfg.persist_best_effort);
}
