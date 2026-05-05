use std::fs;
use std::path::Path;

use serde_json::Value;

pub(super) use super::{clear_valkey_prefix, unique_agentic_prefix, wendao_cmd, write_file};

mod promoted_links_materialize_in_neighbors_and_related;
mod promoted_overlay_is_isolated_by_key_prefix;
mod promoted_overlay_resolves_mixed_alias_forms;
mod provisional_links_are_isolated_before_promotion;

fn write_agentic_config(
    config_path: &Path,
    prefix: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(
        config_path,
        format!(
            "link_graph:\n  cache:\n    valkey_url: \"redis://127.0.0.1:6379/0\"\n    key_prefix: \"{prefix}\"\n  agentic:\n    suggested_link:\n      max_entries: 64\n      ttl_seconds: null\n"
        ),
    )?;
    Ok(())
}

fn run_wendao_ok(
    root: Option<&Path>,
    config_path: &Path,
    args: &[&str],
    context: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut command = wendao_cmd();
    if let Some(root_path) = root {
        command.arg("--root").arg(root_path);
    }
    let output = command.arg("--conf").arg(config_path).args(args).output()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{context}: {stderr}");
    Ok(())
}

fn run_wendao_json(
    root: Option<&Path>,
    config_path: &Path,
    args: &[&str],
    context: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut command = wendao_cmd();
    if let Some(root_path) = root {
        command.arg("--root").arg(root_path);
    }
    let output = command.arg("--conf").arg(config_path).args(args).output()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{context}: {stderr}");
    let stdout = String::from_utf8(output.stdout)?;
    Ok(serde_json::from_str(&stdout)?)
}

fn assert_promoted_overlay_applied(payload: &Value) -> Result<(), Box<dyn std::error::Error>> {
    let overlay = payload
        .get("promoted_overlay")
        .ok_or("missing promoted_overlay telemetry")?;
    assert_eq!(overlay.get("applied").and_then(Value::as_bool), Some(true));
    assert_eq!(
        overlay.get("source").and_then(Value::as_str),
        Some("valkey.suggested_link_recent_latest")
    );
    Ok(())
}

fn assert_verbose_overlay(payload: &Value) -> Result<(), Box<dyn std::error::Error>> {
    assert_promoted_overlay_applied(payload)?;
    assert!(
        payload
            .get("phases")
            .and_then(Value::as_array)
            .is_some_and(|rows| rows.iter().any(|row| {
                row.get("phase").and_then(Value::as_str) == Some("link_graph.overlay.promoted")
            }))
    );
    assert!(
        payload
            .get("monitor")
            .and_then(|row| row.get("bottlenecks"))
            .and_then(|row| row.get("slowest_phase"))
            .is_some()
    );
    Ok(())
}
