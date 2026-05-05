use crate::studio::arrow_types::{LanceArray, LanceStringArray, LanceUInt64Array};

use crate::studio::vfs::flight_content::{
    VfsContentResponse, vfs_content_response_batch, vfs_content_response_flight_app_metadata,
};

#[test]
fn vfs_content_response_batch_preserves_payload_fields() {
    let batch = vfs_content_response_batch(&VfsContentResponse {
        path: "main/docs/index.md".to_string(),
        content_type: "text/plain".to_string(),
        content: "# Index".to_string(),
        modified: 42,
    })
    .unwrap_or_else(|error| panic!("build VFS content batch: {error}"));

    assert_eq!(batch.num_rows(), 1);
    let Some(content_column) = batch.column_by_name("content") else {
        panic!("content column");
    };
    let Some(contents) = content_column.as_any().downcast_ref::<LanceStringArray>() else {
        panic!("content column type");
    };
    assert_eq!(contents.value(0), "# Index");

    let Some(modified_column) = batch.column_by_name("modified") else {
        panic!("modified column");
    };
    let Some(modified) = modified_column.as_any().downcast_ref::<LanceUInt64Array>() else {
        panic!("modified column type");
    };
    assert_eq!(modified.value(0), 42);
}

#[test]
fn vfs_content_response_metadata_preserves_summary_fields() {
    let metadata = vfs_content_response_flight_app_metadata(&VfsContentResponse {
        path: "main/docs/index.md".to_string(),
        content_type: "text/plain".to_string(),
        content: "# Index".to_string(),
        modified: 42,
    })
    .unwrap_or_else(|error| panic!("encode VFS content metadata: {error}"));

    let payload: serde_json::Value = serde_json::from_slice(&metadata)
        .unwrap_or_else(|error| panic!("metadata should decode: {error}"));
    assert_eq!(payload["path"], "main/docs/index.md");
    assert_eq!(payload["contentType"], "text/plain");
    assert_eq!(payload["modified"], 42);
}
