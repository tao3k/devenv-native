use std::fs;

use tempfile::TempDir;
use xiuxian_qianji::{
    WorkdirMarkdownSurface, WorkdirVisibleSurfaceKind, build_workdir_check_follow_up_query,
    check_workdir, query_workdir_markdown_payload, show_workdir,
};

use super::create_semantic_workdir;

#[test]
fn workdir_semantic_surface_checks_valid() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_semantic_workdir(&temp_dir);

    let report = check_workdir(&workdir)
        .unwrap_or_else(|error| panic!("semantic work surface should check: {error}"));
    assert!(report.is_valid());

    let show = show_workdir(&workdir)
        .unwrap_or_else(|error| panic!("semantic work surface should show: {error}"));
    assert!(
        show.surfaces
            .iter()
            .any(|surface| surface.surface == "semantic"
                && surface.kind == WorkdirVisibleSurfaceKind::Directory)
    );
}
#[test]
fn workdir_semantic_follow_up_query_targets_semantic_surface() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_semantic_workdir(&temp_dir);

    fs::remove_file(workdir.join("semantic/objects/component/demo.md"))
        .unwrap_or_else(|error| panic!("should remove semantic object: {error}"));

    let report = check_workdir(&workdir)
        .unwrap_or_else(|error| panic!("invalid semantic surface should still report: {error}"));
    let follow_up = build_workdir_check_follow_up_query(&report)
        .unwrap_or_else(|| panic!("failing semantic report should derive follow-up query"));

    assert_eq!(follow_up.surfaces, vec![WorkdirMarkdownSurface::Semantic]);
    assert_eq!(
        follow_up.query_text,
        "select path, surface, surface_kind, heading_path, skeleton \
from markdown \
where surface = 'semantic' \
order by surface, path, heading_path"
    );
}
#[test]
fn workdir_semantic_change_intent_follow_up_query_targets_semantic_surface() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_semantic_workdir(&temp_dir);

    fs::remove_file(workdir.join("semantic/change-intents/demo-change.md"))
        .unwrap_or_else(|error| panic!("should remove semantic change intent: {error}"));

    let report = check_workdir(&workdir)
        .unwrap_or_else(|error| panic!("invalid semantic surface should still report: {error}"));
    let follow_up = build_workdir_check_follow_up_query(&report)
        .unwrap_or_else(|| panic!("failing semantic report should derive follow-up query"));

    assert_eq!(follow_up.surfaces, vec![WorkdirMarkdownSurface::Semantic]);
    assert_eq!(
        follow_up.query_text,
        "select path, surface, surface_kind, heading_path, skeleton \
from markdown \
where surface = 'semantic' \
order by surface, path, heading_path"
    );
}
#[tokio::test]
async fn workdir_semantic_query_surface_returns_sql_payload() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_semantic_workdir(&temp_dir);

    let payload = query_workdir_markdown_payload(
        &workdir,
        "select path, surface, surface_kind, heading_path from markdown where surface = 'semantic' order by path, heading_path",
    )
    .await
    .unwrap_or_else(|error| panic!("semantic workdir SQL payload should resolve: {error}"));

    assert!(
        payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(|row| row.get("path").and_then(serde_json::Value::as_str)
                == Some("semantic/objects/component/demo.md"))
    );
    assert!(
        payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(
                |row| row.get("surface_kind").and_then(serde_json::Value::as_str)
                    == Some("semantic_object")
            )
    );
    assert!(
        payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(|row| row.get("path").and_then(serde_json::Value::as_str)
                == Some("semantic/change-intents/demo-change.md"))
    );
    assert!(
        payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(
                |row| row.get("surface_kind").and_then(serde_json::Value::as_str)
                    == Some("semantic_change_intent")
            )
    );
    assert!(
        payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(
                |row| row.get("heading_path").and_then(serde_json::Value::as_str)
                    == Some("Demo Component/Authority")
            )
    );
}
