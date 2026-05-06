use tempfile::TempDir;
use xiuxian_qianji::{
    check_workdir, query_workdir_check_follow_up_payload, query_workdir_markdown_payload,
};

use super::{create_valid_workdir, write_file};

#[tokio::test]
async fn workdir_query_surface_returns_sql_payload() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_valid_workdir(&temp_dir);

    let payload = query_workdir_markdown_payload(
        &workdir,
        "select path, heading_path from markdown where surface = 'plan' order by path, heading_path",
    )
    .await
    .unwrap_or_else(|error| panic!("workdir SQL payload should resolve: {error}"));

    assert_eq!(
        payload.metadata.registered_tables,
        vec!["markdown".to_string()]
    );
    assert_eq!(payload.metadata.registered_table_count, 1);
    assert!(
        payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(|row| row.get("path").and_then(serde_json::Value::as_str) == Some("plan/tasks.md"))
    );
    assert!(
        payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(
                |row| row.get("heading_path").and_then(serde_json::Value::as_str)
                    == Some("Plan/Rust")
            )
    );
}
#[tokio::test]
async fn workdir_check_follow_up_query_returns_surface_bounded_payload() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_valid_workdir(&temp_dir);

    write_file(
        &workdir.join("flowchart.mmd"),
        "flowchart LR\n  plan --> blueprint\n",
    );

    let report = check_workdir(&workdir)
        .unwrap_or_else(|error| panic!("invalid work surface should still report: {error}"));
    let follow_up_payload = query_workdir_check_follow_up_payload(&report)
        .await
        .unwrap_or_else(|error| panic!("follow-up payload should resolve: {error}"))
        .unwrap_or_else(|| panic!("failing report should emit follow-up payload"));

    assert_eq!(
        follow_up_payload.metadata.registered_tables,
        vec!["markdown".to_string()]
    );
    assert!(
        follow_up_payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .all(|row| {
                row.get("surface")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|surface| matches!(surface, "blueprint" | "plan"))
            })
    );
    assert!(
        follow_up_payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(|row| row.get("path").and_then(serde_json::Value::as_str)
                == Some("blueprint/architecture.md"))
    );
    assert!(
        follow_up_payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(|row| row.get("path").and_then(serde_json::Value::as_str)
                == Some("plan/tasks.md"))
    );
}
#[tokio::test]
async fn valid_workdir_has_no_follow_up_payload() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_valid_workdir(&temp_dir);

    let report = check_workdir(&workdir)
        .unwrap_or_else(|error| panic!("valid work surface should check: {error}"));
    let follow_up_payload = query_workdir_check_follow_up_payload(&report)
        .await
        .unwrap_or_else(|error| panic!("valid follow-up lookup should not fail: {error}"));

    assert!(follow_up_payload.is_none());
}
