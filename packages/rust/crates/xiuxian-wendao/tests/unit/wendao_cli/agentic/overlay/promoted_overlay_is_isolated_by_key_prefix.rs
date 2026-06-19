use serde_json::Value;
use tempfile::TempDir;

use super::{
    clear_valkey_prefix, run_wendao_json, run_wendao_ok, unique_agentic_prefix,
    write_agentic_config, write_file,
};

#[test]
fn test_wendao_promoted_overlay_is_isolated_by_key_prefix() -> Result<(), Box<dyn std::error::Error>>
{
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# A\n\nalpha\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# B\n\nbeta\n")?;

    let prefix_a = unique_agentic_prefix();
    let prefix_b = unique_agentic_prefix();
    if clear_valkey_prefix(&prefix_a).is_err() || clear_valkey_prefix(&prefix_b).is_err() {
        return Ok(());
    }

    let config_a = tmp.path().join("wendao.a.yaml");
    let config_b = tmp.path().join("wendao.b.yaml");
    write_agentic_config(&config_a, &prefix_a)?;
    write_agentic_config(&config_b, &prefix_b)?;

    let log_payload = run_wendao_json(
        None,
        &config_a,
        &[
            "agentic",
            "log",
            "docs/a.md",
            "docs/b.md",
            "related_to",
            "--confidence",
            "0.9",
            "--evidence",
            "prefix-a-only",
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
        "prefix isolation test",
    ];
    run_wendao_ok(
        None,
        &config_a,
        &decide_args,
        "wendao agentic decide failed",
    )?;

    let payload_a = run_wendao_json(
        Some(tmp.path()),
        &config_a,
        &["search", "alpha", "--limit", "5"],
        "wendao search for prefix_a failed",
    )?;
    assert_eq!(
        payload_a
            .get("promoted_overlay")
            .and_then(|row| row.get("applied"))
            .and_then(Value::as_bool),
        Some(true)
    );

    let payload_b = run_wendao_json(
        Some(tmp.path()),
        &config_b,
        &["search", "alpha", "--limit", "5"],
        "wendao search for prefix_b failed",
    )?;
    assert_eq!(
        payload_b
            .get("promoted_overlay")
            .and_then(|row| row.get("applied"))
            .and_then(Value::as_bool),
        Some(false)
    );

    clear_valkey_prefix(&prefix_a)?;
    clear_valkey_prefix(&prefix_b)?;
    Ok(())
}
