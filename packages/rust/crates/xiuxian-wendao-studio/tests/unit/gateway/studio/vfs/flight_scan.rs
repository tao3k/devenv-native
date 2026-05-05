use crate::studio::arrow_types::{LanceArray, LanceStringArray};

use crate::studio::vfs::flight_scan::{
    VfsCategory, VfsScanEntry, VfsScanResult, vfs_scan_result_batch,
    vfs_scan_result_flight_app_metadata,
};

fn sample_entry() -> VfsScanEntry {
    VfsScanEntry {
        path: "kernel/docs/alpha.md".to_string(),
        name: "alpha.md".to_string(),
        is_dir: false,
        category: VfsCategory::Doc,
        size: 128,
        modified: 42,
        content_type: Some("text/markdown".to_string()),
        has_frontmatter: true,
        wendao_id: Some("doc:alpha".to_string()),
        project_name: Some("kernel".to_string()),
        root_label: Some("docs".to_string()),
        project_root: Some(".".to_string()),
        project_dirs: Some(vec!["docs".to_string()]),
    }
}

#[test]
fn vfs_scan_result_batch_preserves_entry_fields() {
    let batch = vfs_scan_result_batch(&VfsScanResult {
        entries: vec![sample_entry()],
        file_count: 1,
        dir_count: 0,
        scan_duration_ms: 7,
    })
    .unwrap_or_else(|error| panic!("build VFS scan batch: {error}"));

    assert_eq!(batch.num_rows(), 1);
    let Some(project_dirs_column) = batch.column_by_name("projectDirsJson") else {
        panic!("projectDirsJson column");
    };
    let Some(project_dirs) = project_dirs_column
        .as_any()
        .downcast_ref::<LanceStringArray>()
    else {
        panic!("projectDirsJson column type");
    };
    assert_eq!(project_dirs.value(0), "[\"docs\"]");
}

#[test]
fn vfs_scan_result_metadata_preserves_summary_fields() {
    let metadata = vfs_scan_result_flight_app_metadata(&VfsScanResult {
        entries: vec![sample_entry()],
        file_count: 1,
        dir_count: 0,
        scan_duration_ms: 7,
    })
    .unwrap_or_else(|error| panic!("encode VFS scan metadata: {error}"));

    let payload: serde_json::Value = serde_json::from_slice(&metadata)
        .unwrap_or_else(|error| panic!("metadata should decode: {error}"));
    assert_eq!(payload["fileCount"], 1);
    assert_eq!(payload["dirCount"], 0);
    assert_eq!(payload["scanDurationMs"], 7);
}
