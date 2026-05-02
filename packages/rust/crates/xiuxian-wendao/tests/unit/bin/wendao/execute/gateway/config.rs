use std::fs;

use super::{
    DEFAULT_PORT, GatewayRuntimeTomlConfig, get_webhook_from_config,
    parse_gateway_runtime_from_toml, parse_port_from_toml, parse_webhook_from_toml,
    resolve_config_path, resolve_config_path_with_project_root,
    resolve_config_path_with_project_root_value, resolve_port, resolve_webhook_config,
    resolve_webhook_config_with_lookup,
};
use crate::bin_support::wendao::execute::gateway::tests::support::{
    remove_temp_gateway_config, write_temp_gateway_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_resolve_port_cli_priority() {
    let port = resolve_port(Some(8080), None);
    assert_eq!(port, 8080);
}

#[test]
fn test_resolve_port_default() {
    let port = resolve_port(None, None);
    assert_eq!(port, DEFAULT_PORT);
}

#[test]
fn test_resolve_port_from_cli_config_path() {
    let config_path = write_temp_gateway_config(
        r"
[gateway]
port = 18080
",
    );

    let port = resolve_port(None, Some(config_path.as_path()));
    remove_temp_gateway_config(&config_path);

    assert_eq!(port, 18080);
}

#[test]
fn resolve_config_path_prefers_studio_overlay_when_present() -> TestResult {
    let temp = tempfile::tempdir()?;
    let base_path = temp.path().join("wendao.toml");
    let overlay_path = temp.path().join("wendao.studio.overlay.toml");
    fs::write(&base_path, "[gateway]\nport = 9517\n")?;
    fs::write(
        &overlay_path,
        "imports = [\"wendao.toml\"]\n[gateway]\nport = 9610\n",
    )?;

    let resolved = resolve_config_path(Some(base_path.as_path()))
        .unwrap_or_else(|| panic!("effective config path should resolve"));
    assert_eq!(resolved, overlay_path);
    Ok(())
}

#[test]
fn resolve_config_path_falls_back_to_prj_root_wendao_toml() -> TestResult {
    let temp = tempfile::tempdir()?;
    let workspace_path = temp.path();
    let base_path = workspace_path.join("wendao.toml");
    fs::write(&base_path, "[gateway]\nport = 9517\n")?;

    let resolved = resolve_config_path_with_project_root(None, Some(workspace_path))
        .unwrap_or_else(|| panic!("PRJ_ROOT config path should resolve"));
    assert_eq!(resolved, base_path);
    Ok(())
}

#[test]
fn resolve_config_path_uses_shared_relative_prj_root_resolution() -> TestResult {
    let temp = tempfile::tempdir()?;
    let workspace_path = temp.path().join("workspace");
    let nested_path = workspace_path.join("apps/studio");
    fs::create_dir_all(&nested_path)?;
    let base_path = workspace_path.join("wendao.toml");
    fs::write(&base_path, "[gateway]\nport = 9517\n")?;

    let resolved = resolve_config_path_with_project_root_value(
        None,
        Some("../.."),
        Some(nested_path.as_path()),
    )
    .unwrap_or_else(|| panic!("shared PRJ_ROOT config path should resolve"));
    assert_eq!(resolved.canonicalize()?, base_path.canonicalize()?);
    Ok(())
}

#[test]
fn parse_gateway_config_from_overlay_imports() -> TestResult {
    let temp = tempfile::tempdir()?;
    let base_path = temp.path().join("wendao.toml");
    let overlay_path = temp.path().join("wendao.studio.overlay.toml");
    fs::write(
        &base_path,
        "[gateway]\nport = 9517\nwebhook_url = \"http://127.0.0.1:9000/base\"\n",
    )?;
    fs::write(
        &overlay_path,
        "imports = [\"wendao.toml\"]\n[gateway]\nport = 9610\nwebhook_url = \"http://127.0.0.1:9000/overlay\"\n",
    )?;

    assert_eq!(parse_port_from_toml(&overlay_path), Some(9610));
    let webhook = parse_webhook_from_toml(&overlay_path)
        .unwrap_or_else(|| panic!("webhook config should resolve from overlay"));
    assert_eq!(webhook.url, "http://127.0.0.1:9000/overlay");
    Ok(())
}

#[test]
fn parse_gateway_runtime_from_overlay_imports() -> TestResult {
    let temp = tempfile::tempdir()?;
    let base_path = temp.path().join("wendao.toml");
    let overlay_path = temp.path().join("wendao.studio.overlay.toml");
    fs::write(
        &base_path,
        "[gateway.runtime]\nlisten_backlog = 1024\nstudio_concurrency_limit = 48\n",
    )?;
    fs::write(
        &overlay_path,
        "imports = [\"wendao.toml\"]\n[gateway.runtime]\nstudio_request_timeout_secs = 21\nstudio_concurrency_limit = 64\n",
    )?;

    assert_eq!(
        parse_gateway_runtime_from_toml(&overlay_path),
        Some(GatewayRuntimeTomlConfig {
            listen_backlog: Some(1024),
            studio_concurrency_limit: Some(64),
            studio_request_timeout_secs: Some(21),
        })
    );
    Ok(())
}

#[test]
fn test_webhook_config_from_env() {
    let config = resolve_webhook_config(None);
    assert!(config.url.is_empty());
    assert!(config.secret.is_none());
    assert_eq!(config.timeout_secs, 10);
    assert!(config.retry_on_failure);
}

#[test]
fn test_webhook_config_from_lookup_uses_trimmed_env_values() {
    let config = resolve_webhook_config_with_lookup(None, &|name| match name {
        "WENDAO_WEBHOOK_URL" => Some(" http://127.0.0.1:9999/hooks ".to_string()),
        "WENDAO_WEBHOOK_SECRET" => Some(" top-secret ".to_string()),
        _ => None,
    });

    assert_eq!(config.url, "http://127.0.0.1:9999/hooks");
    assert_eq!(config.secret.as_deref(), Some("top-secret"));
    assert_eq!(config.timeout_secs, 10);
    assert!(config.retry_on_failure);
}

#[test]
fn test_webhook_config_from_lookup_ignores_blank_env_values() {
    let config = resolve_webhook_config_with_lookup(None, &|name| match name {
        "WENDAO_WEBHOOK_URL" | "WENDAO_WEBHOOK_SECRET" => Some("   ".to_string()),
        _ => None,
    });

    assert!(config.url.is_empty());
    assert!(config.secret.is_none());
}

#[test]
fn test_resolve_webhook_config_from_cli_config_path() {
    let config_path = write_temp_gateway_config(
        r#"
[gateway]
webhook_url = "http://127.0.0.1:9999"
webhook_secret = "test-secret"
webhook_enabled = true
"#,
    );

    let config = resolve_webhook_config(Some(config_path.as_path()));
    remove_temp_gateway_config(&config_path);

    assert_eq!(config.url, "http://127.0.0.1:9999");
    assert_eq!(config.secret.as_deref(), Some("test-secret"));
    assert_eq!(config.timeout_secs, 10);
}

#[test]
fn test_resolve_webhook_config_prefers_toml_over_env_fallback() {
    let config_path = write_temp_gateway_config(
        r#"
[gateway]
webhook_url = "http://127.0.0.1:9999"
webhook_secret = "test-secret"
webhook_enabled = true
"#,
    );

    let config =
        resolve_webhook_config_with_lookup(Some(config_path.as_path()), &|name| match name {
            "WENDAO_WEBHOOK_URL" => Some("http://127.0.0.1:7777/hooks".to_string()),
            "WENDAO_WEBHOOK_SECRET" => Some("env-secret".to_string()),
            _ => None,
        });
    remove_temp_gateway_config(&config_path);

    assert_eq!(config.url, "http://127.0.0.1:9999");
    assert_eq!(config.secret.as_deref(), Some("test-secret"));
}

#[test]
fn test_disabled_webhook_config_is_ignored() {
    let config_path = write_temp_gateway_config(
        r#"
[gateway]
webhook_url = "http://127.0.0.1:9999"
webhook_enabled = false
"#,
    );

    let config = get_webhook_from_config(Some(config_path.as_path()));
    remove_temp_gateway_config(&config_path);

    assert!(config.is_none());
}

#[test]
fn test_parse_port_from_toml_content() {
    let content = r"
[gateway]
port = 8080
";
    let mut found_port = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("port")
            && let Some(eq_pos) = line.find('=')
        {
            let value = line[eq_pos + 1..].trim().trim_matches('"');
            if let Ok(port) = value.parse::<u16>() {
                assert_eq!(port, 8080);
                found_port = true;
            }
        }
    }
    assert!(found_port);
}
