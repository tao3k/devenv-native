use crate::test_wendao_cli::support::wendao_cmd;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub(crate) fn write_agentic_execution_config(
    config_path: &Path,
    prefix: &str,
    max_candidates: usize,
    max_pairs_per_worker: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(
        config_path,
        format!(
            "link_graph:\n  cache:\n    valkey_url: \"redis://127.0.0.1:6379/0\"\n    key_prefix: \"{prefix}\"\n  agentic:\n    suggested_link:\n      max_entries: 64\n      ttl_seconds: null\n    expansion:\n      max_workers: 1\n      max_candidates: {max_candidates}\n      max_pairs_per_worker: {max_pairs_per_worker}\n      time_budget_ms: 1000.0\n    execution:\n      worker_time_budget_ms: 1000.0\n      persist_suggestions_default: true\n      persist_retry_attempts: 2\n      idempotency_scan_limit: 64\n      relation: \"related_to\"\n      agent_id: \"wendao-agentic-architect\"\n      evidence_prefix: \"agentic expansion bridge candidate\"\n"
        ),
    )?;
    Ok(())
}

pub(crate) fn run_agentic_run_persist(
    root: &Path,
    config_path: &Path,
    query: &str,
    context: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = wendao_cmd()
        .arg("--root")
        .arg(root)
        .arg("--conf")
        .arg(config_path)
        .arg("agentic")
        .arg("run")
        .arg("--query")
        .arg(query)
        .arg("--persist")
        .output()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{context}: {stderr}");
    let stdout = String::from_utf8(output.stdout)?;
    Ok(serde_json::from_str(&stdout)?)
}

pub(crate) fn run_agentic_recent_json(
    config_path: &Path,
    state: &str,
    limit: usize,
    context: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let output = wendao_cmd()
        .arg("--conf")
        .arg(config_path)
        .arg("agentic")
        .arg("recent")
        .arg("--latest")
        .arg("--state")
        .arg(state)
        .arg("--limit")
        .arg(limit.to_string())
        .output()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{context}: {stderr}");
    let stdout = String::from_utf8(output.stdout)?;
    Ok(serde_json::from_str(&stdout)?)
}
