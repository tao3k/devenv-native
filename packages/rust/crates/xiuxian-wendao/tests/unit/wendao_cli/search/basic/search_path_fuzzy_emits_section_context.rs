use serde_json::Value;
use tempfile::TempDir;

use super::{wendao_cmd, write_file};

#[test]
fn test_wendao_search_path_fuzzy_emits_section_context() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/architecture/graph.md"),
        "# Architecture\n\n## Graph Engine\n\nDetails.\n",
    )?;
    write_file(
        &tmp.path().join("docs/misc.md"),
        "# Misc\n\nGraph mention.\n",
    )?;

    let output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("search")
        .arg("architecture graph engine")
        .arg("--limit")
        .arg("5")
        .arg("--match-strategy")
        .arg("path_fuzzy")
        .output()?;

    assert!(
        output.status.success(),
        "wendao search with path_fuzzy failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout)?;
    let payload: Value = serde_json::from_str(&stdout)?;
    assert_eq!(
        payload.get("match_strategy").and_then(Value::as_str),
        Some("path_fuzzy")
    );
    let rows = payload
        .get("results")
        .and_then(Value::as_array)
        .ok_or("missing results")?;
    assert!(!rows.is_empty());
    assert_eq!(
        rows.first()
            .and_then(|row| row.get("path"))
            .and_then(Value::as_str),
        Some("docs/architecture/graph.md")
    );
    assert!(
        rows.first()
            .and_then(|row| row.get("best_section"))
            .and_then(Value::as_str)
            .is_some_and(|v| v.contains("Graph Engine"))
    );
    Ok(())
}
