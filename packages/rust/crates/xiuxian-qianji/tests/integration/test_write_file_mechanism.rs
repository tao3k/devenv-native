//! Integration tests for native `write_file` mechanism.

use serde_json::json;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;
use xiuxian_qianji::executors::WriteFileMechanism;
use xiuxian_qianji::{FlowInstruction, QianjiMechanism};

#[tokio::test]
async fn write_file_creates_parent_dirs_and_persists_content() {
    let workspace = tempdir().unwrap_or_else(|error| panic!("tempdir should work: {error}"));
    let destination = workspace
        .path()
        .join("nested")
        .join("persona")
        .join("teacher.md");
    let destination_text = destination.display().to_string();

    let mechanism = WriteFileMechanism {
        path: "{{output_path}}".to_string(),
        content: "{{document_body}}".to_string(),
        output_key: "write_result".to_string(),
    };

    let context = json!({
        "output_path": destination_text,
        "document_body": "# Teacher\n\nDiscipline builds consistency.\n"
    });

    let output = mechanism
        .execute(&context)
        .await
        .unwrap_or_else(|error| panic!("write_file should succeed: {error}"));

    assert!(matches!(output.instruction, FlowInstruction::Continue));

    let persisted = fs::read_to_string(destination.as_path())
        .unwrap_or_else(|error| panic!("written file should be readable: {error}"));
    assert_eq!(persisted, "# Teacher\n\nDiscipline builds consistency.\n");
    assert_eq!(
        output.data["write_result"]["path"].as_str(),
        Some(destination.display().to_string().as_str())
    );
    assert_eq!(
        output.data["write_result"]["bytes_written"].as_u64(),
        Some(persisted.len() as u64)
    );
}

#[tokio::test]
async fn write_file_supports_nested_context_placeholders() {
    let workspace = tempdir().unwrap_or_else(|error| panic!("tempdir should work: {error}"));
    let destination = workspace.path().join("nested").join("avatar.md");
    let destination_text = destination.display().to_string();

    let mechanism = WriteFileMechanism {
        path: "{{paths.output_file}}".to_string(),
        content: "{{artifact.body}}".to_string(),
        output_key: "write_result".to_string(),
    };

    let context = json!({
        "paths": {
            "output_file": destination_text
        },
        "artifact": {
            "body": "Nested placeholder rendering works.\n"
        }
    });

    let output = mechanism
        .execute(&context)
        .await
        .unwrap_or_else(|error| panic!("write_file should support nested placeholders: {error}"));

    assert!(matches!(output.instruction, FlowInstruction::Continue));
    let persisted = fs::read_to_string(destination.as_path())
        .unwrap_or_else(|error| panic!("written file should be readable: {error}"));
    assert_eq!(persisted, "Nested placeholder rendering works.\n");
}

#[tokio::test]
async fn write_file_rejects_path_escape_when_project_root_is_set() {
    let workspace = tempdir().unwrap_or_else(|error| panic!("tempdir should work: {error}"));
    let mechanism = WriteFileMechanism {
        path: "../escape.md".to_string(),
        content: "blocked".to_string(),
        output_key: "write_result".to_string(),
    };
    let context = json!({
        "project_root": workspace.path().display().to_string()
    });

    let result = mechanism.execute(&context).await;
    assert!(result.is_err(), "write_file should block root escape");
    let error_text = result
        .err()
        .unwrap_or_else(|| "write_file failed with unknown error".to_string());
    assert!(
        error_text.contains("escapes root directory"),
        "unexpected error: {error_text}"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn write_file_returns_error_on_permission_denied_directory() {
    let workspace = tempdir().unwrap_or_else(|error| panic!("tempdir should work: {error}"));
    let readonly_dir = workspace.path().join("readonly");
    fs::create_dir_all(readonly_dir.as_path())
        .unwrap_or_else(|error| panic!("create_dir_all should work: {error}"));

    let mut readonly_permissions = fs::metadata(readonly_dir.as_path())
        .unwrap_or_else(|error| panic!("metadata should work: {error}"))
        .permissions();
    readonly_permissions.set_mode(0o555);
    fs::set_permissions(readonly_dir.as_path(), readonly_permissions)
        .unwrap_or_else(|error| panic!("set_permissions should work: {error}"));

    let blocked_file = readonly_dir.join("blocked.md");
    let mechanism = WriteFileMechanism {
        path: "{{output_path}}".to_string(),
        content: "blocked".to_string(),
        output_key: "write_result".to_string(),
    };
    let context = json!({
        "output_path": blocked_file.display().to_string()
    });

    let result = mechanism.execute(&context).await;

    let mut restore_permissions = fs::metadata(readonly_dir.as_path())
        .unwrap_or_else(|error| panic!("metadata for restore should work: {error}"))
        .permissions();
    restore_permissions.set_mode(0o755);
    fs::set_permissions(readonly_dir.as_path(), restore_permissions)
        .unwrap_or_else(|error| panic!("restore permissions should work: {error}"));

    assert!(
        result.is_err(),
        "write_file should fail in readonly directory"
    );
    let error_text = result
        .err()
        .unwrap_or_else(|| "write_file failed with unknown error".to_string());
    assert!(
        error_text.contains("write_file failed to write")
            || error_text.to_lowercase().contains("permission denied")
    );
}
