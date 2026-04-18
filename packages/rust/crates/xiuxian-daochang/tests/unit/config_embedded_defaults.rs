//! Embedded-defaults bootstrap tests for `xiuxian-daochang` runtime settings.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use tempfile::TempDir;
use xiuxian_daochang::load_runtime_settings;

const CHILD_ENV_KEY: &str = "XIUXIAN_DAOCHANG_CONFIG_EMBEDDED_DEFAULTS_CHILD";

fn write_user_settings(root: &Path, user_toml: &str) -> Result<()> {
    let user_path = root.join(".config/xiuxian-artisan-workshop/xiuxian.toml");
    if let Some(parent) = user_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(user_path, user_toml)?;
    Ok(())
}

#[test]
fn load_runtime_settings_uses_embedded_defaults_when_system_file_missing() -> Result<()> {
    let root = TempDir::new()?;
    write_user_settings(root.path(), "")?;
    let test_binary = std::env::current_exe().context("resolve current test binary path")?;
    let output = Command::new(test_binary)
        .arg("--exact")
        .arg("embedded_defaults_child_probe")
        .arg("--nocapture")
        .env(CHILD_ENV_KEY, "1")
        .env("PRJ_ROOT", root.path())
        .env("PRJ_CONFIG_HOME", root.path().join(".config"))
        .output()
        .context("spawn embedded defaults child probe")?;
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "embedded defaults child probe failed exit_code={:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            stdout,
            stderr
        );
    }
    Ok(())
}

#[test]
fn embedded_defaults_child_probe() {
    if std::env::var(CHILD_ENV_KEY).ok().as_deref() != Some("1") {
        return;
    }

    let settings = load_runtime_settings();
    assert_eq!(settings.inference.provider.as_deref(), Some("minimax"));
    assert_eq!(settings.inference.model.as_deref(), Some("MiniMax-M2.5"));
    assert_eq!(
        settings.inference.api_key.as_deref(),
        Some("MINIMAX_API_KEY")
    );
}
