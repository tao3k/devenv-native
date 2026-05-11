use std::path::PathBuf;
use std::sync::Arc;

use crate::repo_index::RepoCodeDocument;
use crate::search::service::tests::support::{service_test_manifest_keyspace, temp_dir};
use crate::search::{SearchMaintenancePolicy, SearchPlaneService};

pub(super) fn test_service() -> SearchPlaneService {
    let temp_dir = temp_dir();
    SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        service_test_manifest_keyspace(),
        SearchMaintenancePolicy::default(),
    )
}

pub(super) fn repo_document(
    path: &str,
    contents: &str,
    size_bytes: u64,
    modified_unix_ms: u64,
) -> RepoCodeDocument {
    RepoCodeDocument {
        path: path.to_string().into(),
        language: Some("rust".to_string()),
        contents: Arc::<str>::from(contents),
        size_bytes,
        modified_unix_ms,
    }
}
