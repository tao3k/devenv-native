use serde_json::Value;
use tempfile::TempDir;

use super::{parse_success_json, wendao_cmd, write_file};

#[test]
fn test_wendao_hmas_validate_command() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(
        &tmp.path().join("thread.md"),
        r#"
#### [CONCLUSION]
```json
{
  "requirement_id": "REQ-CLI-1",
  "summary": "CLI validator smoke test",
  "confidence_score": 0.9,
  "hard_constraints_checked": ["RULE"]
}
```

#### [DIGITAL THREAD]
```json
{
  "requirement_id": "REQ-CLI-1",
  "source_nodes_accessed": [{"node_id": "note-1"}],
  "hard_constraints_checked": ["RULE"],
  "confidence_score": 0.9
}
```
"#,
    )?;

    let output = wendao_cmd()
        .arg("hmas")
        .arg("validate")
        .arg("--file")
        .arg(tmp.path().join("thread.md"))
        .output()?;
    let payload = parse_success_json(output, "wendao hmas validate failed")?;
    assert_eq!(payload.get("valid").and_then(Value::as_bool), Some(true));
    assert_eq!(
        payload.get("digital_thread_count").and_then(Value::as_u64),
        Some(1)
    );
    Ok(())
}
