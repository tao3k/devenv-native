use serde_json::Value;
use tempfile::TempDir;

use super::{
    clear_valkey_prefix, run_agentic_decide_promoted, run_agentic_decisions,
    run_agentic_log_default, run_agentic_recent_provisional, unique_agentic_prefix,
    write_agentic_base_config,
};

#[test]
fn test_wendao_agentic_log_recent_decide_flow() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let prefix = unique_agentic_prefix();
    if clear_valkey_prefix(&prefix).is_err() {
        return Ok(());
    }

    let config_path = tmp.path().join("wendao.yaml");
    write_agentic_base_config(&config_path, &prefix)?;

    let logged = run_agentic_log_default(&config_path)?;
    let suggestion_id = logged
        .get("suggestion_id")
        .and_then(Value::as_str)
        .ok_or("missing suggestion_id")?;
    assert_eq!(
        logged.get("promotion_state").and_then(Value::as_str),
        Some("provisional")
    );

    let recent_rows = run_agentic_recent_provisional(&config_path)?;
    let recent_rows = recent_rows
        .as_array()
        .ok_or("recent payload must be array")?;
    assert_eq!(recent_rows.len(), 1);
    assert_eq!(
        recent_rows[0].get("suggestion_id").and_then(Value::as_str),
        Some(suggestion_id)
    );

    let decide_payload = run_agentic_decide_promoted(&config_path, suggestion_id)?;
    assert_eq!(
        decide_payload
            .get("suggestion")
            .and_then(|row| row.get("promotion_state"))
            .and_then(Value::as_str),
        Some("promoted")
    );

    let decisions_rows = run_agentic_decisions(&config_path)?;
    let decisions_rows = decisions_rows
        .as_array()
        .ok_or("decisions payload must be array")?;
    assert_eq!(decisions_rows.len(), 1);
    assert_eq!(
        decisions_rows[0]
            .get("suggestion_id")
            .and_then(Value::as_str),
        Some(suggestion_id)
    );
    assert_eq!(
        decisions_rows[0]
            .get("target_state")
            .and_then(Value::as_str),
        Some("promoted")
    );

    clear_valkey_prefix(&prefix)?;
    Ok(())
}
