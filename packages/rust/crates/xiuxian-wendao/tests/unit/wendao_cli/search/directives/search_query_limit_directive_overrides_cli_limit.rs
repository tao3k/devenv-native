use serde_json::Value;
use tempfile::TempDir;

use super::{wendao_cmd, write_file};

#[test]
fn test_wendao_search_query_limit_directive_overrides_cli_limit()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# A\n\nshared keyword\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# B\n\nshared keyword\n")?;
    write_file(&tmp.path().join("docs/c.md"), "# C\n\nshared keyword\n")?;

    let output = wendao_cmd()
        .arg("--root")
        .arg(tmp.path())
        .arg("search")
        .arg("query:shared keyword limit:1 sort:path_asc")
        .arg("--limit")
        .arg("10")
        .output()?;

    assert!(
        output.status.success(),
        "wendao search with query limit directive failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let payload: Value = serde_json::from_str(&String::from_utf8(output.stdout)?)?;
    assert_eq!(
        payload.get("query").and_then(Value::as_str),
        Some("shared keyword")
    );
    assert_eq!(payload.get("limit").and_then(Value::as_u64), Some(1));
    let rows = payload
        .get("results")
        .and_then(Value::as_array)
        .ok_or("missing results")?;
    assert_eq!(rows.len(), 1);
    Ok(())
}
