//! Integration tests for `wendao_refresh` mechanism.

use std::fs;
use tempfile::TempDir;
use xiuxian_qianji::executors::WendaoRefreshMechanism;
use xiuxian_qianji::{FlowInstruction, QianjiMechanism};

fn write_file(path: &std::path::Path, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

xiuxian_testing::crate_test_policy_harness!();

#[tokio::test]
async fn wendao_refresh_returns_delta_when_changed_paths_present()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let changed_file = tmp.path().join("docs").join("alpha.md");
    write_file(changed_file.as_path(), "# Alpha\n\n[[beta]]\n")?;
    write_file(&tmp.path().join("docs").join("beta.md"), "# Beta\n")?;

    let mechanism = WendaoRefreshMechanism {
        output_key: "refresh_report".to_string(),
        changed_paths_key: "changed_paths".to_string(),
        root_dir_key: None,
        root_dir: Some(tmp.path().display().to_string()),
        force_full: false,
        prefer_incremental: true,
        allow_full_fallback: true,
        full_rebuild_threshold: None,
        include_dirs: Vec::new(),
        excluded_dirs: Vec::new(),
    };

    let output = mechanism
        .execute(&serde_json::json!({
            "changed_paths": [changed_file.display().to_string()]
        }))
        .await?;

    assert!(matches!(output.instruction, FlowInstruction::Continue));
    let Some(payload) = output
        .data
        .get("refresh_report")
        .and_then(serde_json::Value::as_object)
    else {
        panic!("refresh_report payload missing");
    };
    let Some(mode) = payload.get("mode").and_then(serde_json::Value::as_str) else {
        panic!("refresh_report.mode missing");
    };
    assert_eq!(mode, "delta");
    let Some(changed_count) = payload
        .get("changed_count")
        .and_then(serde_json::Value::as_u64)
    else {
        panic!("refresh_report.changed_count missing");
    };
    assert_eq!(changed_count, 1);

    Ok(())
}

#[tokio::test]
async fn wendao_refresh_returns_noop_when_no_changed_paths()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    write_file(&tmp.path().join("docs").join("alpha.md"), "# Alpha\n")?;

    let mechanism = WendaoRefreshMechanism {
        output_key: "refresh_report".to_string(),
        changed_paths_key: "changed_paths".to_string(),
        root_dir_key: None,
        root_dir: Some(tmp.path().display().to_string()),
        force_full: false,
        prefer_incremental: true,
        allow_full_fallback: true,
        full_rebuild_threshold: None,
        include_dirs: Vec::new(),
        excluded_dirs: Vec::new(),
    };

    let output = mechanism.execute(&serde_json::json!({})).await?;

    assert!(matches!(output.instruction, FlowInstruction::Continue));
    let mode = output
        .data
        .get("refresh_report")
        .and_then(serde_json::Value::as_object)
        .and_then(|payload| payload.get("mode"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    assert_eq!(mode, "noop");
    Ok(())
}
