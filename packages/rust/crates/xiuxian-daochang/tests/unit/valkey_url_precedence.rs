//! Valkey URL precedence tests for runtime settings and environment fallback.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use tempfile::TempDir;
use xiuxian_daochang::{SessionGate, SessionStore};

const CHILD_ENV_KEY: &str = "XIUXIAN_DAOCHANG_VALKEY_PRECEDENCE_CHILD";
const CHILD_CASE_KEY: &str = "XIUXIAN_DAOCHANG_VALKEY_PRECEDENCE_CASE";

fn write_runtime_settings(root: &Path, system_toml: &str) -> Result<()> {
    let system_path =
        root.join("packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml");
    let user_path = root.join(".config/xiuxian-artisan-workshop/xiuxian.toml");
    if let Some(parent) = system_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if let Some(parent) = user_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(system_path, system_toml)?;
    std::fs::write(user_path, "")?;
    Ok(())
}

fn run_child_case(
    root: &Path,
    case: &str,
    valkey_url: Option<&str>,
    namespaced_valkey_url: Option<&str>,
) -> Result<()> {
    let test_binary = std::env::current_exe().context("resolve current test binary path")?;
    let mut command = Command::new(test_binary);
    command
        .arg("--exact")
        .arg("valkey_url_precedence_child_probe")
        .arg("--nocapture")
        .env(CHILD_ENV_KEY, "1")
        .env(CHILD_CASE_KEY, case)
        .env("PRJ_ROOT", root)
        .env("PRJ_CONFIG_HOME", root.join(".config"));
    if let Some(url) = valkey_url {
        command.env("VALKEY_URL", url);
    } else {
        command.env_remove("VALKEY_URL");
    }
    if let Some(url) = namespaced_valkey_url {
        command.env("XIUXIAN_WENDAO_VALKEY_URL", url);
    } else {
        command.env_remove("XIUXIAN_WENDAO_VALKEY_URL");
    }
    let output = command
        .output()
        .with_context(|| format!("spawn child probe for case={case}"))?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "child probe failed for case={case} exit_code={:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            stdout,
            stderr
        );
    }
    Ok(())
}

#[test]
fn valkey_url_resolution_prefers_settings_and_keeps_env_fallback() -> Result<()> {
    let case_settings_first = TempDir::new()?;
    write_runtime_settings(
        case_settings_first.path(),
        r#"
[session]
valkey_url = "redis://127.0.0.1:6379/0"

[telegram]
foreground_session_gate_backend = "valkey"
"#,
    )?;
    run_child_case(
        case_settings_first.path(),
        "settings_first",
        Some("://invalid-url-should-not-win"),
        Some("://invalid-url-should-not-win"),
    )?;

    let case_env_fallback = TempDir::new()?;
    write_runtime_settings(
        case_env_fallback.path(),
        r#"
[telegram]
foreground_session_gate_backend = "valkey"
"#,
    )?;
    run_child_case(
        case_env_fallback.path(),
        "env_fallback",
        Some("redis://127.0.0.1:6379/1"),
        None,
    )?;

    let case_namespaced_env_wins = TempDir::new()?;
    write_runtime_settings(
        case_namespaced_env_wins.path(),
        r#"
[telegram]
foreground_session_gate_backend = "valkey"
"#,
    )?;
    run_child_case(
        case_namespaced_env_wins.path(),
        "namespaced_env_wins",
        Some("://invalid-legacy-env"),
        Some("redis://127.0.0.1:6379/2"),
    )?;

    Ok(())
}

#[test]
fn valkey_url_precedence_child_probe() -> Result<()> {
    if std::env::var(CHILD_ENV_KEY).ok().as_deref() != Some("1") {
        return Ok(());
    }

    let case = std::env::var(CHILD_CASE_KEY).unwrap_or_else(|_| "unknown".to_string());
    match case.as_str() {
        "settings_first" | "env_fallback" | "namespaced_env_wins" => {}
        other => bail!("unknown child probe case: {other}"),
    }

    let gate = SessionGate::from_env().context("construct SessionGate from env/settings")?;
    assert_eq!(gate.backend_name(), "valkey");

    let _store = SessionStore::new().context("construct SessionStore from env/settings")?;
    Ok(())
}
