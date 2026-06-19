use std::fs;
use std::path::Path;

use serde_json::Value;

pub(crate) use super::{clear_valkey_prefix, unique_agentic_prefix, wendao_cmd, write_file};

mod execution;
mod log_flow;
mod overlay;
mod planning;
fn write_agentic_base_config(
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

fn run_agentic_json(
    config_path: &Path,
    args: &[&str],
    context: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = wendao_cmd()
        .arg("--conf")
        .arg(config_path)
        .args(args)
        .output()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{context}: {stderr}");
    let stdout = String::from_utf8(output.stdout)?;
    Ok(serde_json::from_str(&stdout)?)
}

fn run_agentic_log_default(config_path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    run_agentic_json(
        config_path,
        &[
            "agentic",
            "log",
            "docs/a.md",
            "docs/b.md",
            "implements",
            "--confidence",
            "0.8",
            "--evidence",
            "bridge found",
            "--agent-id",
            "wendao-agentic-architect",
            "--created-at-unix",
            "1700000300",
        ],
        "wendao agentic log failed",
    )
}

fn run_agentic_recent_provisional(config_path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    run_agentic_json(
        config_path,
        &[
            "agentic",
            "recent",
            "--limit",
            "10",
            "--latest",
            "--state",
            "provisional",
        ],
        "wendao agentic recent failed",
    )
}

fn run_agentic_decide_promoted(
    config_path: &Path,
    suggestion_id: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let args = vec![
        "agentic",
        "decide",
        suggestion_id,
        "--target-state",
        "promoted",
        "--decided-by",
        "omega-gate",
        "--reason",
        "validated",
        "--decided-at-unix",
        "1700000310",
    ];
    run_agentic_json(config_path, &args, "wendao agentic decide failed")
}

fn run_agentic_decisions(config_path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    run_agentic_json(
        config_path,
        &["agentic", "decisions", "--limit", "10"],
        "wendao agentic decisions failed",
    )
}
