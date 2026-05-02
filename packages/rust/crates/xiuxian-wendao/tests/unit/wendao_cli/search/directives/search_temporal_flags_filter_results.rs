use serde_json::Value;
use tempfile::TempDir;

use super::{wendao_cmd, write_file};

#[test]
fn test_wendao_search_temporal_flags_filter_results() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("docs/a.md"),
        "---\ncreated: 2024-01-01\nmodified: 2024-01-05\n---\n# A\n",
    )?;
    write_file(
        &tmp.path().join("docs/b.md"),
        "---\ncreated: 2024-01-03\nmodified: 2024-01-02\n---\n# B\n",
    )?;
    write_file(
        &tmp.path().join("docs/c.md"),
        "---\ncreated: 2024-01-10\nmodified: 2024-01-12\n---\n# C\n",
    )?;

    let output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("search")
        .arg(".md")
        .arg("--limit")
        .arg("10")
        .arg("--sort-term")
        .arg("created_asc")
        .arg("--created-after")
        .arg("1704153600")
        .arg("--created-before")
        .arg("1704758400")
        .output()?;

    assert!(
        output.status.success(),
        "wendao search with temporal flags failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_str(&String::from_utf8(output.stdout)?)?;
    assert_eq!(
        payload.get("created_after").and_then(Value::as_i64),
        Some(1_704_153_600)
    );
    assert_eq!(
        payload.get("created_before").and_then(Value::as_i64),
        Some(1_704_758_400)
    );
    let sort_terms = payload
        .get("sort_terms")
        .and_then(Value::as_array)
        .ok_or("missing sort_terms")?;
    assert_eq!(sort_terms.len(), 1);
    assert_eq!(
        sort_terms[0].get("field").and_then(Value::as_str),
        Some("created")
    );
    assert_eq!(
        sort_terms[0].get("order").and_then(Value::as_str),
        Some("asc")
    );
    let rows = payload
        .get("results")
        .and_then(Value::as_array)
        .ok_or("missing results")?;
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows.first()
            .and_then(|row| row.get("path"))
            .and_then(Value::as_str),
        Some("docs/b.md")
    );
    Ok(())
}
