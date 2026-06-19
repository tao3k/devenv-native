use serde_json::Value;
use tempfile::TempDir;

use super::{
    clear_valkey_prefix, run_wendao_json, run_wendao_ok, unique_agentic_prefix,
    write_agentic_config, write_file,
};

#[test]
fn test_wendao_promoted_overlay_resolves_mixed_alias_forms()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# A\n\nalpha\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# B\n\nbeta\n")?;

    let prefix = unique_agentic_prefix();
    if clear_valkey_prefix(&prefix).is_err() {
        return Ok(());
    }

    let config_path = tmp.path().join("wendao.yaml");
    write_agentic_config(&config_path, &prefix)?;

    let log_payload = run_wendao_json(
        None,
        &config_path,
        &[
            "agentic",
            "log",
            "a",
            "docs/b.md",
            "related_to",
            "--confidence",
            "0.93",
            "--evidence",
            "mixed-alias-forms",
            "--agent-id",
            "wendao-agentic-architect",
        ],
        "wendao agentic log failed",
    )?;
    let suggestion_id = log_payload
        .get("suggestion_id")
        .and_then(Value::as_str)
        .ok_or("missing suggestion_id")?;

    let decide_args = vec![
        "agentic",
        "decide",
        suggestion_id,
        "--target-state",
        "promoted",
        "--decided-by",
        "omega-gate",
        "--reason",
        "alias mapping verification",
    ];
    run_wendao_ok(
        None,
        &config_path,
        &decide_args,
        "wendao agentic decide failed",
    )?;

    let payload = run_wendao_json(
        Some(tmp.path()),
        &config_path,
        &[
            "neighbors",
            "docs/a.md",
            "--direction",
            "outgoing",
            "--hops",
            "1",
            "--limit",
            "10",
            "--verbose",
        ],
        "wendao neighbors --verbose failed",
    )?;
    let rows = payload
        .get("results")
        .and_then(Value::as_array)
        .ok_or("missing neighbors results")?;
    assert!(
        rows.iter()
            .any(|row| row.get("stem").and_then(Value::as_str) == Some("b")),
        "expected promoted edge to resolve mixed alias forms: payload={payload}"
    );
    assert_eq!(
        payload
            .get("promoted_overlay")
            .and_then(|row| row.get("applied"))
            .and_then(Value::as_bool),
        Some(true)
    );

    clear_valkey_prefix(&prefix)?;
    Ok(())
}
