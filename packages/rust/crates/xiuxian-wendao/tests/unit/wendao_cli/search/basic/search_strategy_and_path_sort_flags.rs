use serde_json::Value;
use tempfile::TempDir;

use super::{wendao_cmd, write_file};

#[test]
fn test_wendao_search_strategy_and_path_sort_flags() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("notes/zeta.md"), "# Zeta\n\nkeyword\n")?;
    write_file(&tmp.path().join("notes/alpha.md"), "# Alpha\n\nkeyword\n")?;

    let output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("search")
        .arg(".md")
        .arg("--limit")
        .arg("5")
        .arg("--match-strategy")
        .arg("fts")
        .arg("--sort-term")
        .arg("path_asc")
        .output()?;

    assert!(
        output.status.success(),
        "wendao search with strategy/path sort failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout)?;
    let payload: Value = serde_json::from_str(&stdout)?;
    assert_eq!(
        payload.get("match_strategy").and_then(Value::as_str),
        Some("fts")
    );
    let sort_terms = payload
        .get("sort_terms")
        .and_then(Value::as_array)
        .ok_or("missing sort_terms")?;
    assert_eq!(sort_terms.len(), 1);
    assert_eq!(
        sort_terms[0].get("field").and_then(Value::as_str),
        Some("path")
    );
    assert_eq!(
        sort_terms[0].get("order").and_then(Value::as_str),
        Some("asc")
    );
    let rows = payload
        .get("results")
        .and_then(Value::as_array)
        .ok_or("missing results")?;
    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows.first()
            .and_then(|row| row.get("path"))
            .and_then(Value::as_str),
        Some("notes/alpha.md")
    );
    Ok(())
}
