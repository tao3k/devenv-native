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
use serde_json::Value;
use tempfile::TempDir;

use super::{clear_valkey_prefix, unique_agentic_prefix, wendao_cmd, write_file};

#[test]
fn test_wendao_agentic_run_can_persist_suggestions() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# A\n\nalpha\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# B\n\nalpha\n")?;

    let prefix = unique_agentic_prefix();
    if clear_valkey_prefix(&prefix).is_err() {
        return Ok(());
    }

    let config_path = tmp.path().join("wendao.yaml");
    fs::write(
        &config_path,
        format!(
            "link_graph:\n  cache:\n    valkey_url: \"redis://127.0.0.1:6379/0\"\n    key_prefix: \"{prefix}\"\n  agentic:\n    suggested_link:\n      max_entries: 64\n      ttl_seconds: null\n    expansion:\n      max_workers: 1\n      max_candidates: 2\n      max_pairs_per_worker: 1\n      time_budget_ms: 1000.0\n    execution:\n      worker_time_budget_ms: 1000.0\n      persist_suggestions_default: true\n      persist_retry_attempts: 2\n      idempotency_scan_limit: 64\n      relation: \"related_to\"\n      agent_id: \"wendao-agentic-architect\"\n      evidence_prefix: \"agentic expansion bridge candidate\"\n"
        ),
    )?;

    let run_output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("--conf")
        .arg(&config_path)
        .arg("agentic")
        .arg("run")
        .arg("--query")
        .arg("alpha")
        .arg("--persist")
        .output()?;
    assert!(
        run_output.status.success(),
        "wendao agentic run persist failed: {}",
        String::from_utf8_lossy(&run_output.stderr)
    );
    let run_stdout = String::from_utf8(run_output.stdout)?;
    let run_payload: Value = serde_json::from_str(&run_stdout)?;
    let persisted = run_payload
        .get("persisted_proposals")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    assert!(persisted >= 1);
    assert_eq!(
        run_payload.get("failed_proposals").and_then(Value::as_u64),
        Some(0)
    );

    let run_output_2 = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("--conf")
        .arg(&config_path)
        .arg("agentic")
        .arg("run")
        .arg("--query")
        .arg("alpha")
        .arg("--persist")
        .output()?;
    assert!(
        run_output_2.status.success(),
        "wendao second agentic run persist failed: {}",
        String::from_utf8_lossy(&run_output_2.stderr)
    );
    let run_stdout_2 = String::from_utf8(run_output_2.stdout)?;
    let run_payload_2: Value = serde_json::from_str(&run_stdout_2)?;
    assert_eq!(
        run_payload_2
            .get("persisted_proposals")
            .and_then(Value::as_u64),
        Some(0)
    );
    assert_eq!(
        run_payload_2
            .get("skipped_duplicates")
            .and_then(Value::as_u64),
        Some(1)
    );

    let recent_output = wendao_cmd()
        .arg("--conf")
        .arg(&config_path)
        .arg("agentic")
        .arg("recent")
        .arg("--latest")
        .arg("--state")
        .arg("provisional")
        .arg("--limit")
        .arg("10")
        .output()?;
    assert!(
        recent_output.status.success(),
        "wendao agentic recent after run failed: {}",
        String::from_utf8_lossy(&recent_output.stderr)
    );
    let recent_stdout = String::from_utf8(recent_output.stdout)?;
    let rows: Value = serde_json::from_str(&recent_stdout)?;
    let rows = rows.as_array().ok_or("recent payload must be array")?;
    assert!(!rows.is_empty());
    assert!(
        rows.iter()
            .all(|row| row.get("promotion_state").and_then(Value::as_str) == Some("provisional"))
    );

    clear_valkey_prefix(&prefix)?;
    Ok(())
}
