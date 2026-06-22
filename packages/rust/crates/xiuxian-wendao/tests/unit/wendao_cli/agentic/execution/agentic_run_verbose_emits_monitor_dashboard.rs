use crate::test_wendao_cli::support::{wendao_cmd, write_file};
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_wendao_agentic_run_verbose_emits_monitor_dashboard()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# A\n\nalpha\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# B\n\nalpha\n")?;
    write_file(&tmp.path().join("docs/c.md"), "# C\n\nbeta\n")?;

    let config_path = tmp.path().join("wendao.yaml");
    fs::write(
        &config_path,
        "link_graph:\n  agentic:\n    expansion:\n      max_workers: 1\n      max_candidates: 3\n      max_pairs_per_worker: 1\n      time_budget_ms: 1000.0\n    execution:\n      worker_time_budget_ms: 1000.0\n      persist_suggestions_default: false\n      relation: \"related_to\"\n      agent_id: \"wendao-agentic-architect\"\n      evidence_prefix: \"agentic expansion bridge candidate\"\n",
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
        .arg("--verbose")
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "wendao agentic run --verbose failed: {stderr}"
    );

    let stdout = String::from_utf8(output.stdout)?;
    let payload: Value = serde_json::from_str(&stdout)?;
    let phases = payload
        .get("phases")
        .and_then(Value::as_array)
        .ok_or("missing top-level phases in verbose payload")?;
    assert!(!phases.is_empty());
    assert!(
        phases
            .iter()
            .any(|row| row.get("phase").and_then(Value::as_str) == Some("agentic.plan"))
    );
    assert!(
        phases
            .iter()
            .any(|row| row.get("phase").and_then(Value::as_str) == Some("agentic.worker.total"))
    );

    let monitor = payload
        .get("monitor")
        .ok_or("missing monitor in verbose payload")?;
    assert_eq!(
        monitor
            .get("overview")
            .and_then(|value| value.get("worker_runs"))
            .and_then(Value::as_u64),
        Some(1)
    );
    assert!(
        monitor
            .get("bottlenecks")
            .and_then(|value| value.get("slowest_phase"))
            .is_some()
    );

    Ok(())
}
