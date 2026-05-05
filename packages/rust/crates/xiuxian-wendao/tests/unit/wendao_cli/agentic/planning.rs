#![allow(
    missing_docs,
    clippy::doc_markdown,
    clippy::implicit_clone,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::manual_string_new,
    clippy::needless_raw_string_hashes,
    clippy::format_push_string,
    clippy::unnecessary_to_owned,
    clippy::too_many_lines
)]
use std::fs;

use serde_json::Value;
use tempfile::TempDir;

use super::{wendao_cmd, write_file};

#[test]
fn test_wendao_agentic_plan_uses_config_runtime_budgets() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# A\n\nalpha\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# B\n\nalpha\n")?;
    write_file(&tmp.path().join("docs/c.md"), "# C\n\nbeta\n")?;
    write_file(&tmp.path().join("docs/d.md"), "# D\n\ngamma\n")?;

    let config_path = tmp.path().join("wendao.yaml");
    fs::write(
        &config_path,
        "link_graph:\n  agentic:\n    expansion:\n      max_workers: 1\n      max_candidates: 4\n      max_pairs_per_worker: 1\n      time_budget_ms: 1000.0\n",
    )?;

    let output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("--conf")
        .arg(&config_path)
        .arg("agentic")
        .arg("plan")
        .arg("--query")
        .arg("alpha")
        .output()?;

    assert!(
        output.status.success(),
        "wendao agentic plan failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout)?;
    let payload: Value = serde_json::from_str(&stdout)?;
    assert_eq!(
        payload
            .get("config")
            .and_then(|value| value.get("max_workers"))
            .and_then(Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload
            .get("config")
            .and_then(|value| value.get("max_pairs_per_worker"))
            .and_then(Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload
            .get("workers")
            .and_then(Value::as_array)
            .map(std::vec::Vec::len),
        Some(1)
    );
    assert_eq!(
        payload.get("selected_pairs").and_then(Value::as_u64),
        Some(1)
    );
    Ok(())
}

#[test]
fn test_wendao_agentic_run_uses_config_runtime_budgets_and_telemetry()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# A\n\nalpha\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# B\n\nalpha\n")?;
    write_file(&tmp.path().join("docs/c.md"), "# C\n\nbeta\n")?;

    let config_path = tmp.path().join("wendao.yaml");
    fs::write(
        &config_path,
        "link_graph:\n  agentic:\n    expansion:\n      max_workers: 1\n      max_candidates: 3\n      max_pairs_per_worker: 1\n      time_budget_ms: 1000.0\n    execution:\n      worker_time_budget_ms: 1000.0\n      persist_suggestions_default: false\n      relation: \"related_to\"\n      agent_id: \"qianhuan-architect\"\n      evidence_prefix: \"agentic expansion bridge candidate\"\n",
    )?;

    let output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("--conf")
        .arg(&config_path)
        .arg("agentic")
        .arg("run")
        .arg("--query")
        .arg("alpha")
        .output()?;

    assert!(
        output.status.success(),
        "wendao agentic run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout)?;
    let payload: Value = serde_json::from_str(&stdout)?;
    assert_eq!(
        payload
            .get("config")
            .and_then(|value| value.get("expansion"))
            .and_then(|value| value.get("max_workers"))
            .and_then(Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload
            .get("config")
            .and_then(|value| value.get("worker_time_budget_ms"))
            .and_then(Value::as_f64),
        Some(1000.0)
    );
    assert_eq!(
        payload
            .get("config")
            .and_then(|value| value.get("persist_suggestions"))
            .and_then(Value::as_bool),
        Some(false)
    );
    assert_eq!(
        payload
            .get("config")
            .and_then(|value| value.get("persist_retry_attempts"))
            .and_then(Value::as_u64),
        Some(2)
    );
    assert_eq!(
        payload
            .get("config")
            .and_then(|value| value.get("idempotency_scan_limit"))
            .and_then(Value::as_u64),
        Some(2000)
    );
    assert_eq!(
        payload.get("prepared_proposals").and_then(Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload.get("persisted_proposals").and_then(Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload.get("failed_proposals").and_then(Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload.get("persist_attempts").and_then(Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload.get("skipped_duplicates").and_then(Value::as_u64),
        Some(0)
    );
    let worker_runs = payload
        .get("worker_runs")
        .and_then(Value::as_array)
        .ok_or("missing worker_runs")?;
    assert_eq!(worker_runs.len(), 1);
    assert!(
        worker_runs[0]
            .get("estimated_prompt_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            > 0
    );
    let phases = worker_runs[0]
        .get("phases")
        .and_then(Value::as_array)
        .ok_or("missing phases")?;
    assert!(phases.iter().any(|phase| {
        phase.get("phase").and_then(Value::as_str) == Some("worker.total")
            && phase.get("item_count").and_then(Value::as_u64) == Some(1)
    }));

    Ok(())
}
