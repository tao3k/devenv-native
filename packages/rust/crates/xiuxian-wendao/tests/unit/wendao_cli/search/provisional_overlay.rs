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

use super::{clear_valkey_prefix, unique_agentic_prefix, wendao_cmd, write_file};

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
    fs::write(
        &config_path,
        format!(
            "link_graph:\n  cache:\n    valkey_url: \"redis://127.0.0.1:6379/0\"\n    key_prefix: \"{prefix}\"\n  agentic:\n    suggested_link:\n      max_entries: 64\n      ttl_seconds: null\n"
        ),
    )?;

    let log_output = wendao_cmd()
        .arg("--conf")
        .arg(&config_path)
        .arg("agentic")
        .arg("log")
        .arg("a")
        .arg("docs/b.md")
        .arg("related_to")
        .arg("--confidence")
        .arg("0.93")
        .arg("--evidence")
        .arg("mixed-alias-forms")
        .arg("--agent-id")
        .arg("wendao-agentic-architect")
        .output()?;
    assert!(
        log_output.status.success(),
        "wendao agentic log failed: {}",
        String::from_utf8_lossy(&log_output.stderr)
    );
    let log_payload: Value = serde_json::from_str(&String::from_utf8(log_output.stdout)?)?;
    let suggestion_id = log_payload
        .get("suggestion_id")
        .and_then(Value::as_str)
        .ok_or("missing suggestion_id")?
        .to_string();

    let decide_output = wendao_cmd()
        .arg("--conf")
        .arg(&config_path)
        .arg("agentic")
        .arg("decide")
        .arg(&suggestion_id)
        .arg("--target-state")
        .arg("promoted")
        .arg("--decided-by")
        .arg("omega-gate")
        .arg("--reason")
        .arg("alias mapping verification")
        .output()?;
    assert!(
        decide_output.status.success(),
        "wendao agentic decide failed: {}",
        String::from_utf8_lossy(&decide_output.stderr)
    );

    let neighbors_output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("--conf")
        .arg(&config_path)
        .arg("neighbors")
        .arg("docs/a.md")
        .arg("--direction")
        .arg("outgoing")
        .arg("--hops")
        .arg("1")
        .arg("--limit")
        .arg("10")
        .arg("--verbose")
        .output()?;
    assert!(
        neighbors_output.status.success(),
        "wendao neighbors --verbose failed: {}",
        String::from_utf8_lossy(&neighbors_output.stderr)
    );
    let payload: Value = serde_json::from_str(&String::from_utf8(neighbors_output.stdout)?)?;
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
