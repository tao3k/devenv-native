use serde_json::Value;
use tempfile::TempDir;

use super::{parse_success_json, wendao_cmd, write_file};

#[test]
fn test_wendao_allows_global_root_after_subcommand() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs/a.md"), "# Alpha\n\n[[b]]\n")?;
    write_file(&tmp.path().join("docs/b.md"), "# Beta\n\n[[a]]\n")?;

    let output = wendao_cmd()
        .arg("search")
        .arg("alpha")
        .arg("--root")
        .arg(tmp.path())
        .arg("--limit")
        .arg("2")
        .output()?;
    let payload = parse_success_json(output, "wendao search with trailing --root failed")?;
    assert_eq!(payload.get("query").and_then(Value::as_str), Some("alpha"));
    assert!(
        payload
            .get("results")
            .and_then(Value::as_array)
            .is_some_and(|rows| !rows.is_empty())
    );
    Ok(())
}
