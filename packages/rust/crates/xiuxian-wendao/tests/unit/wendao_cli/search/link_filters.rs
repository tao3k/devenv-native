use serde_json::Value;
use tempfile::TempDir;

use super::{wendao_cmd, write_file};

#[test]
fn test_wendao_search_link_filters_flags() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# A\n\n[[b]]\n")?;
    write_file(&tmp.path().join("docs/c.md"), "# C\n\n[[b]]\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# B\n\nNo links.\n")?;

    let output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("search")
        .arg(".md")
        .arg("--limit")
        .arg("10")
        .arg("--link-to")
        .arg("b")
        .arg("--sort-term")
        .arg("path_asc")
        .output()?;

    assert!(
        output.status.success(),
        "wendao search with link filters failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout)?;
    let payload: Value = serde_json::from_str(&stdout)?;
    let filters = payload.get("filters").ok_or("missing filters payload")?;
    assert_eq!(
        filters
            .get("link_to")
            .and_then(|row| row.get("seeds"))
            .and_then(Value::as_array)
            .map(std::vec::Vec::len),
        Some(1)
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
        Some("docs/a.md")
    );
    assert_eq!(
        rows.get(1)
            .and_then(|row| row.get("path"))
            .and_then(Value::as_str),
        Some("docs/c.md")
    );
    Ok(())
}

#[test]
fn test_wendao_search_related_ppr_flags() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# A\n\n[[b]]\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# B\n\n[[c]]\n")?;
    write_file(&tmp.path().join("docs/c.md"), "# C\n\n[[d]]\n")?;
    write_file(&tmp.path().join("docs/d.md"), "# D\n\nNo links.\n")?;

    let output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("search")
        .arg(".md")
        .arg("--limit")
        .arg("10")
        .arg("--related")
        .arg("b")
        .arg("--max-distance")
        .arg("2")
        .arg("--related-ppr-alpha")
        .arg("0.9")
        .arg("--related-ppr-max-iter")
        .arg("64")
        .arg("--related-ppr-tol")
        .arg("1e-6")
        .arg("--related-ppr-subgraph-mode")
        .arg("force")
        .arg("--sort-term")
        .arg("path_asc")
        .output()?;

    assert!(
        output.status.success(),
        "wendao search with related ppr flags failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_str(&String::from_utf8(output.stdout)?)?;
    let related = payload
        .get("filters")
        .and_then(|row| row.get("related"))
        .ok_or("missing related filter payload")?;
    assert_eq!(
        related
            .get("seeds")
            .and_then(Value::as_array)
            .map(std::vec::Vec::len),
        Some(1)
    );
    assert_eq!(related.get("max_distance").and_then(Value::as_u64), Some(2));
    let ppr = related.get("ppr").ok_or("missing related ppr payload")?;
    assert_eq!(ppr.get("alpha").and_then(Value::as_f64), Some(0.9));
    assert_eq!(ppr.get("max_iter").and_then(Value::as_u64), Some(64));
    assert_eq!(ppr.get("tol").and_then(Value::as_f64), Some(1e-6));
    assert_eq!(
        ppr.get("subgraph_mode").and_then(Value::as_str),
        Some("force")
    );

    Ok(())
}
