use std::fs;
use tempfile::tempdir;
use xiuxian_daochang::load_xiuxian_config_from_bases;

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_unified_config_loading_priority() -> TestResult {
    let tmp = tempdir()?;
    let system_base = tmp.path().join("system");
    let user_base = tmp.path().join("user");
    fs::create_dir_all(&system_base)?;
    fs::create_dir_all(&user_base)?;

    fs::write(
        user_base.join("xiuxian.toml"),
        r#"
[wendao.zhixing]
notebook_path = "/custom/unified"
"#,
    )?;

    let config = load_xiuxian_config_from_bases(&system_base, &user_base);
    assert_eq!(
        config.wendao.zhixing.notebook_path.as_deref(),
        Some("/custom/unified")
    );
    Ok(())
}

#[test]
fn test_modular_wendao_fallback() -> TestResult {
    let tmp = tempdir()?;
    let system_base = tmp.path().join("system");
    let user_base = tmp.path().join("user");
    fs::create_dir_all(&system_base)?;
    fs::create_dir_all(&user_base)?;

    fs::write(
        user_base.join("wendao.shared.toml"),
        r#"
[zhixing]
notebook_path = "/modular/fallback"
"#,
    )?;
    fs::write(
        user_base.join("wendao.toml"),
        r#"
imports = ["wendao.shared.toml"]

[zhixing]
# notebook_path is sourced from wendao.shared.toml
"#,
    )?;

    let config = load_xiuxian_config_from_bases(&system_base, &user_base);

    assert_eq!(
        config.wendao.zhixing.notebook_path.as_deref(),
        Some("/modular/fallback")
    );
    Ok(())
}

#[test]
fn test_wendao_gateway_config_loading() -> TestResult {
    let tmp = tempdir()?;
    let system_base = tmp.path().join("system");
    let user_base = tmp.path().join("user");
    fs::create_dir_all(&system_base)?;
    fs::create_dir_all(&user_base)?;

    fs::write(
        user_base.join("xiuxian.toml"),
        r#"
[wendao_gateway]
query_endpoint = "http://127.0.0.1:18093/query"
default_project_root = "/repo/default"

[wendao_gateway.session_project_roots]
"discord:123" = "/repo/discord"
"telegram:456" = "/repo/telegram"
"#,
    )?;

    let config = load_xiuxian_config_from_bases(&system_base, &user_base);

    assert_eq!(
        config.wendao_gateway.query_endpoint.as_deref(),
        Some("http://127.0.0.1:18093/query")
    );
    assert_eq!(
        config.wendao_gateway.default_project_root.as_deref(),
        Some("/repo/default")
    );
    assert_eq!(
        config
            .wendao_gateway
            .session_project_roots
            .get("discord:123")
            .map(String::as_str),
        Some("/repo/discord")
    );
    assert_eq!(
        config
            .wendao_gateway
            .session_project_roots
            .get("telegram:456")
            .map(String::as_str),
        Some("/repo/telegram")
    );
    Ok(())
}
